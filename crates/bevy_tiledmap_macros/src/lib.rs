//! Procedural macros for `bevy_tiled`.
//!
//! This crate provides the `TiledClass` derive macro for automatic component
//! registration and property deserialization.

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DeriveInput, Fields, Lit, Meta, MetaNameValue, Type, Variant,
    parse_macro_input, punctuated::Punctuated, token::Comma,
};

/// Get the path tokens for bevy_tiledmap crate (either umbrella or core).
/// Returns tokens for accessing properties module and other re-exports.
fn get_crate_paths() -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    // Try to find the umbrella crate first
    if let Ok(found) = crate_name("bevy_tiledmap") {
        let base = match found {
            // For FoundCrate::Itself, we still use the crate name because:
            // - Examples within the crate use `bevy_tiledmap::` not `crate::`
            // - The macro generates code that runs in user code, not lib code
            FoundCrate::Itself | FoundCrate::Name(_) => quote!(::bevy_tiledmap),
        };
        // Umbrella crate: properties are at bevy_tiledmap::core::properties
        (
            quote!(#base::core::properties),
            quote!(#base::inventory),
            quote!(#base::tiled),
        )
    } else if let Ok(found) = crate_name("bevy_tiledmap_core") {
        // Fall back to core crate directly
        let base = match found {
            // Same reasoning as above
            FoundCrate::Itself | FoundCrate::Name(_) => quote!(::bevy_tiledmap_core),
        };
        // Core crate: properties are at bevy_tiledmap_core::properties
        // inventory and tiled must be root-level deps in this case
        (
            quote!(#base::properties),
            quote!(::inventory),
            quote!(::tiled),
        )
    } else {
        // Fallback - assume umbrella crate
        (
            quote!(::bevy_tiledmap::core::properties),
            quote!(::bevy_tiledmap::inventory),
            quote!(::bevy_tiledmap::tiled),
        )
    }
}

/// Derive macro for registering a type as a Tiled custom class.
///
/// This macro generates:
/// - An inventory submission to register the type at compile time
/// - A deserialization function that converts Tiled properties to the component
/// - Validation that the type implements `Component + Reflect`
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use bevy_tiledmap_macros::TiledClass;
///
/// #[derive(Component, Reflect, TiledClass)]
/// #[tiled(name = "game::Door")]
/// struct Door {
///     locked: bool,
///     key_id: Option<u32>,
/// }
/// ```
///
/// # Attributes
///
/// - `#[tiled(name = "...")]` - Set the exported name for Tiled (required)
/// - `#[tiled(default = ...)]` - Default value if property is missing (field-level)
/// - `#[tiled(skip)]` - Don't deserialize this field (field-level)
#[proc_macro_derive(TiledClass, attributes(tiled))]
pub fn derive_tiled_class(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_tiled_class_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Crate paths for code generation
struct CratePaths {
    properties: proc_macro2::TokenStream,
    inventory: proc_macro2::TokenStream,
    tiled: proc_macro2::TokenStream,
}

fn derive_tiled_class_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let type_name = &input.ident;

    // Get crate paths once
    let (properties, inventory, tiled) = get_crate_paths();
    let paths = CratePaths {
        properties,
        inventory,
        tiled,
    };

    // Parse #[tiled(name = "...")] attribute
    let tiled_name = parse_tiled_name_attr(&input.attrs)?;

    // Handle structs or enums
    match &input.data {
        Data::Struct(data) => {
            // Handle struct (including unit structs)
            match &data.fields {
                Fields::Named(fields) => handle_struct(type_name, &tiled_name, &fields.named, &paths),
                Fields::Unit => handle_unit_struct(type_name, &tiled_name, &paths),
                Fields::Unnamed(_) => Err(syn::Error::new_spanned(
                    type_name,
                    "TiledClass does not support tuple structs",
                )),
            }
        }
        Data::Enum(data) => {
            // Handle enum
            handle_enum(type_name, &tiled_name, data, &input.attrs, &paths)
        }
        _ => Err(syn::Error::new_spanned(
            type_name,
            "TiledClass can only be derived for structs or enums",
        )),
    }
}

fn handle_struct(
    struct_name: &syn::Ident,
    tiled_name: &str,
    fields: &Punctuated<syn::Field, Comma>,
    paths: &CratePaths,
) -> syn::Result<TokenStream> {
    let properties = &paths.properties;

    // Generate field deserialization code and metadata
    // We need two versions: one for Result context, one for Option context
    let mut field_inits_result = Vec::new();
    let mut field_inits_option = Vec::new();
    let mut field_metadata = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check for #[tiled(skip)]
        if has_skip_attr(&field.attrs) {
            // Use Default::default() for skipped fields
            let skip_init = quote! {
                #field_name: ::std::default::Default::default()
            };
            field_inits_result.push(skip_init.clone());
            field_inits_option.push(skip_init);
            // Skipped fields don't appear in metadata
            continue;
        }

        // Check for #[tiled(default = ...)]
        let default_value = parse_default_attr(&field.attrs)?;

        // Generate field metadata for JSON export
        let field_name_str = field_name.to_string();
        let tiled_type = map_rust_type_to_tiled(field_type, paths);
        let default_expr = generate_default_value_expr(field_type, &default_value, paths)?;

        field_metadata.push(quote! {
            #properties::TiledFieldInfo {
                name: #field_name_str,
                tiled_type: #tiled_type,
                default_value: #default_expr,
            }
        });

        // Get the actual type (unwrap Option if needed)
        let actual_type = extract_option_inner_type(field_type).unwrap_or(field_type);

        // Check if this is a Handle<T> field
        if is_handle_type(actual_type) {
            // Handle<T> fields use AssetServer to load
            let is_optional = extract_option_inner_type(field_type).is_some();
            let tiled = &paths.tiled;

            if is_optional {
                // Option<Handle<T>>: load if path exists, None otherwise
                // Paths are already normalized during map loading (Layer 1)
                field_inits_result.push(quote! {
                    #field_name: __properties.get(#field_name_str)
                        .and_then(|v| match v {
                            #tiled::PropertyValue::StringValue(s) if !s.is_empty() => {
                                __asset_server.map(|server| server.load(s.clone()))
                            }
                            #tiled::PropertyValue::FileValue(s) if !s.is_empty() => {
                                __asset_server.map(|server| server.load(s.clone()))
                            }
                            _ => ::std::option::Option::None,
                        })
                });
                // For Option context (FromTiledProperty), no asset server available
                field_inits_option.push(quote! {
                    #field_name: ::std::option::Option::None
                });
            } else {
                // Required Handle<T>: must have path and asset server
                // Paths are already normalized during map loading (Layer 1)
                field_inits_result.push(quote! {
                    #field_name: {
                        let path = __properties.get(#field_name_str)
                            .and_then(|v| match v {
                                #tiled::PropertyValue::StringValue(s) => ::std::option::Option::Some(s.clone()),
                                #tiled::PropertyValue::FileValue(s) => ::std::option::Option::Some(s.clone()),
                                _ => ::std::option::Option::None,
                            })
                            .ok_or_else(|| ::std::format!("Missing asset path for field '{}'", #field_name_str))?;
                        let server = __asset_server
                            .ok_or_else(|| ::std::format!("AssetServer required for field '{}' but not provided", #field_name_str))?;
                        server.load(path)
                    }
                });
                // For Option context (FromTiledProperty), Handle fields require AssetServer which is not available.
                // Return None immediately to indicate deserialization cannot proceed.
                field_inits_option.push(quote! {
                    #field_name: {
                        // Handle<T> fields require AssetServer which FromTiledProperty does not have
                        return ::std::option::Option::None;
                        #[allow(unreachable_code)]
                        ::std::default::Default::default()
                    }
                });
            }
            continue;
        }

        // Generate property access code for Result context (used in __tiled_from_properties)
        // and Option context (used in FromTiledProperty impl)
        if let Some(inner_type) = extract_option_inner_type(field_type) {
            // Optional fields: extract inner type T from Option<T>
            let init = quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#inner_type as #properties::FromTiledProperty>::from_property(v))
            };
            field_inits_result.push(init.clone());
            field_inits_option.push(init);
        } else if let Some(ref default) = default_value {
            // Fields with defaults use the default if missing
            let init = quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))
                    .unwrap_or(#default)
            };
            field_inits_result.push(init.clone());
            field_inits_option.push(init);
        } else {
            // Required fields - different handling for Result vs Option context
            field_inits_result.push(quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))
                    .ok_or_else(|| format!("Missing required property '{}'", #field_name_str))?
            });
            // For Option context, use ? which propagates None
            field_inits_option.push(quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))?
            });
        }
    }

    // Generate static field metadata array (uppercase for lint compliance)
    let fields_array_name =
        quote::format_ident!("__TILED_FIELDS_{}", struct_name.to_string().to_uppercase());

    let inventory = &paths.inventory;
    let tiled = &paths.tiled;

    // Generate the complete implementation
    let expanded = quote! {
        // Static array of field metadata for JSON export
        #[doc(hidden)]
        static #fields_array_name: &[#properties::TiledFieldInfo] = &[
            #(#field_metadata),*
        ];

        // Submit to inventory for compile-time registration
        #inventory::submit! {
            #properties::TiledClassInfo {
                type_id: ::std::any::TypeId::of::<#struct_name>(),
                name: #tiled_name,
                fields: #fields_array_name,
                from_properties: #struct_name::__tiled_from_properties,
            }
        }

        impl #struct_name {
            #[doc(hidden)]
            fn __tiled_from_properties(
                __properties: &#tiled::Properties,
                __asset_server: ::std::option::Option<&::bevy::asset::AssetServer>,
            ) -> ::std::result::Result<::std::boxed::Box<dyn ::bevy::reflect::Reflect>, ::std::string::String> {
                let instance = Self {
                    #(#field_inits_result),*
                };

                Ok(::std::boxed::Box::new(instance))
            }
        }

        // Implement FromTiledProperty to allow nested class fields
        impl #properties::FromTiledProperty for #struct_name {
            fn from_property(value: &#tiled::PropertyValue) -> ::std::option::Option<Self> {
                match value {
                    #tiled::PropertyValue::ClassValue { properties: __properties, .. } => {
                        let instance = Self {
                            #(#field_inits_option),*
                        };
                        ::std::option::Option::Some(instance)
                    }
                    _ => ::std::option::Option::None,
                }
            }
        }
    };

    Ok(expanded.into())
}

/// Handle unit struct (no fields) - used as marker components
fn handle_unit_struct(struct_name: &syn::Ident, tiled_name: &str, paths: &CratePaths) -> syn::Result<TokenStream> {
    // Generate static field metadata array (empty for unit structs)
    let fields_array_name =
        quote::format_ident!("__TILED_FIELDS_{}", struct_name.to_string().to_uppercase());

    let properties = &paths.properties;
    let inventory = &paths.inventory;
    let tiled = &paths.tiled;

    let expanded = quote! {
        // Static array of field metadata (empty for unit struct)
        #[doc(hidden)]
        static #fields_array_name: &[#properties::TiledFieldInfo] = &[];

        // Submit to inventory for compile-time registration
        #inventory::submit! {
            #properties::TiledClassInfo {
                type_id: ::std::any::TypeId::of::<#struct_name>(),
                name: #tiled_name,
                fields: #fields_array_name,
                from_properties: #struct_name::__tiled_from_properties,
            }
        }

        impl #struct_name {
            #[doc(hidden)]
            fn __tiled_from_properties(
                _properties: &#tiled::Properties,
                _asset_server: ::std::option::Option<&::bevy::asset::AssetServer>,
            ) -> ::std::result::Result<::std::boxed::Box<dyn ::bevy::reflect::Reflect>, ::std::string::String> {
                Ok(::std::boxed::Box::new(Self))
            }
        }

        // Implement FromTiledProperty to allow nested class fields
        impl #properties::FromTiledProperty for #struct_name {
            fn from_property(value: &#tiled::PropertyValue) -> ::std::option::Option<Self> {
                match value {
                    #tiled::PropertyValue::ClassValue { .. } => {
                        ::std::option::Option::Some(Self)
                    }
                    _ => ::std::option::Option::None,
                }
            }
        }
    };

    Ok(expanded.into())
}

fn handle_enum(
    enum_name: &syn::Ident,
    tiled_name: &str,
    data: &DataEnum,
    attrs: &[syn::Attribute],
    paths: &CratePaths,
) -> syn::Result<TokenStream> {
    // Analyze enum variants to determine if unit-only or complex
    let enum_kind = analyze_enum_variants(&data.variants)?;

    // Check for #[tiled(enum = "struct")] attribute
    let enum_format = parse_enum_format_attr(attrs)?;

    match (enum_kind, enum_format) {
        (EnumKind::UnitOnly, EnumFormat::Auto) => {
            // Generate unit-variant enum implementation
            generate_unit_enum_impl(enum_name, tiled_name, &data.variants, paths)
        }
        (EnumKind::Complex, _) | (_, EnumFormat::Struct) => {
            // Generate complex enum implementation (struct/tuple variants)
            let analysis = analyze_enum_variants_detailed(&data.variants)?;
            generate_complex_enum_impl(enum_name, tiled_name, &analysis, paths)
        }
    }
}

#[derive(Debug, PartialEq)]
enum EnumKind {
    UnitOnly,
    Complex,
}

#[derive(Debug, PartialEq)]
enum EnumFormat {
    Auto,
    Struct,
}

/// Information about a single variant analyzed from the enum
#[derive(Clone)]
struct VariantAnalysis {
    /// Variant identifier
    ident: syn::Ident,
    /// Variant name as string
    name: String,
    /// Variant fields (None for unit variants)
    fields: Option<VariantFields>,
    /// Whether this variant has the #[default] attribute
    is_default: bool,
}

/// Variant field information
#[derive(Clone)]
enum VariantFields {
    /// Named fields (struct variant)
    Named(Vec<NamedFieldInfo>),
    /// Unnamed fields (tuple variant)
    Unnamed(Vec<UnnamedFieldInfo>),
}

#[derive(Clone)]
struct NamedFieldInfo {
    ident: syn::Ident,
    ty: Type,
}

#[derive(Clone)]
struct UnnamedFieldInfo {
    index: usize,
    ty: Type,
}

/// Result of analyzing all variants in an enum
struct EnumAnalysis {
    variants: Vec<VariantAnalysis>,
}

/// Analyze enum variants to determine if they're all unit variants
fn analyze_enum_variants(variants: &Punctuated<Variant, Comma>) -> syn::Result<EnumKind> {
    for variant in variants {
        match &variant.fields {
            Fields::Unit => continue,
            Fields::Named(_) | Fields::Unnamed(_) => {
                return Ok(EnumKind::Complex);
            }
        }
    }
    Ok(EnumKind::UnitOnly)
}

/// Analyze enum variants in detail, extracting field metadata and default markers
fn analyze_enum_variants_detailed(
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<EnumAnalysis> {
    let mut analyzed_variants = Vec::new();

    for variant in variants {
        let ident = variant.ident.clone();
        let name = ident.to_string();

        // Check for #[default] attribute
        let is_default = variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("default"));

        // Analyze variant fields
        let fields = match &variant.fields {
            Fields::Unit => None,
            Fields::Named(fields) => {
                let named_fields: Vec<NamedFieldInfo> = fields
                    .named
                    .iter()
                    .map(|f| NamedFieldInfo {
                        ident: f.ident.clone().unwrap(),
                        ty: f.ty.clone(),
                    })
                    .collect();
                Some(VariantFields::Named(named_fields))
            }
            Fields::Unnamed(fields) => {
                let unnamed_fields: Vec<UnnamedFieldInfo> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, f)| UnnamedFieldInfo {
                        index,
                        ty: f.ty.clone(),
                    })
                    .collect();
                Some(VariantFields::Unnamed(unnamed_fields))
            }
        };

        analyzed_variants.push(VariantAnalysis {
            ident,
            name,
            fields,
            is_default,
        });
    }

    Ok(EnumAnalysis {
        variants: analyzed_variants,
    })
}

/// Parse #[tiled(enum = "struct")] attribute
fn parse_enum_format_attr(attrs: &[syn::Attribute]) -> syn::Result<EnumFormat> {
    for attr in attrs {
        if !attr.path().is_ident("tiled") {
            continue;
        }

        if let Meta::List(list) = &attr.meta
            && let Ok(nested) = syn::parse2::<MetaNameValue>(list.tokens.clone())
            && nested.path.is_ident("enum")
            && let syn::Expr::Lit(expr_lit) = &nested.value
            && let Lit::Str(lit_str) = &expr_lit.lit
            && lit_str.value() == "struct"
        {
            return Ok(EnumFormat::Struct);
        }
    }
    Ok(EnumFormat::Auto)
}

/// Generate implementation for unit-variant enum
fn generate_unit_enum_impl(
    enum_name: &syn::Ident,
    tiled_name: &str,
    variants: &Punctuated<Variant, Comma>,
    paths: &CratePaths,
) -> syn::Result<TokenStream> {
    let properties = &paths.properties;
    let inventory = &paths.inventory;
    let tiled = &paths.tiled;

    // Extract variant names
    let variant_names: Vec<String> = variants.iter().map(|v| v.ident.to_string()).collect();

    // Generate match arms for string → enum conversion
    let variant_match_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = variant_ident.to_string();
            quote! {
                #variant_name => Ok(::std::boxed::Box::new(#enum_name::#variant_ident)),
            }
        })
        .collect();

    // Generate match arms for FromTiledProperty
    let from_property_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = variant_ident.to_string();
            quote! {
                #variant_name => ::std::option::Option::Some(Self::#variant_ident),
            }
        })
        .collect();

    // Generate static variants array name
    let variants_array_name = quote::format_ident!(
        "__TILED_ENUM_VARIANTS_{}",
        enum_name.to_string().to_uppercase()
    );

    let expanded = quote! {
        // Static array of variant names
        #[doc(hidden)]
        static #variants_array_name: &[&str] = &[
            #(#variant_names),*
        ];

        // Implement FromTiledProperty for the enum
        impl #properties::FromTiledProperty for #enum_name {
            fn from_property(value: &#tiled::PropertyValue) -> ::std::option::Option<Self> {
                match value {
                    #tiled::PropertyValue::StringValue(s) => {
                        match s.as_str() {
                            #(#from_property_arms)*
                            _ => ::std::option::Option::None,
                        }
                    }
                    _ => ::std::option::Option::None,
                }
            }
        }

        // Submit to inventory for compile-time registration
        #inventory::submit! {
            #properties::TiledEnumInfo {
                type_id: ::std::any::TypeId::of::<#enum_name>(),
                name: #tiled_name,
                kind: #properties::TiledEnumKind::Simple {
                    variants: #variants_array_name,
                    from_string: |s: &str| -> ::std::result::Result<::std::boxed::Box<dyn ::bevy::reflect::Reflect>, ::std::string::String> {
                        match s {
                            #(#variant_match_arms)*
                            _ => ::std::result::Result::Err(
                                ::std::format!("Invalid variant '{}' for enum '{}'", s, #tiled_name)
                            ),
                        }
                    },
                },
                from_property: |value: &#tiled::PropertyValue| -> ::std::result::Result<::std::boxed::Box<dyn ::bevy::reflect::Reflect>, ::std::string::String> {
                    match value {
                        #tiled::PropertyValue::StringValue(s) => {
                            match s.as_str() {
                                #(#variant_match_arms)*
                                _ => ::std::result::Result::Err(
                                    ::std::format!("Invalid variant '{}' for enum '{}'", s, #tiled_name)
                                ),
                            }
                        }
                        _ => ::std::result::Result::Err(
                            ::std::format!("Expected StringValue for simple enum '{}'", #tiled_name)
                        ),
                    }
                },
            }
        }
    };

    Ok(expanded.into())
}

/// Generate implementation for complex enum (with struct/tuple variants)
fn generate_complex_enum_impl(
    enum_name: &syn::Ident,
    tiled_name: &str,
    analysis: &EnumAnalysis,
    paths: &CratePaths,
) -> syn::Result<TokenStream> {
    let properties = &paths.properties;
    let inventory = &paths.inventory;
    let tiled = &paths.tiled;

    // Generate field metadata arrays for each variant
    let variant_metadata_arrays = generate_variant_metadata_arrays(enum_name, &analysis.variants, paths)?;

    // Generate FromTiledProperty implementation
    let from_property_impl =
        generate_complex_from_property_impl(enum_name, tiled_name, &analysis.variants, paths)?;

    // Generate TiledVariantInfo array
    let variant_info_array = generate_variant_info_array(enum_name, &analysis.variants, paths)?;

    // Generate inventory submission
    let inventory_submission = quote! {
        #inventory::submit! {
            #properties::TiledEnumInfo {
                type_id: ::std::any::TypeId::of::<#enum_name>(),
                name: #tiled_name,
                kind: #properties::TiledEnumKind::Complex {
                    variant_info: #variant_info_array,
                },
                from_property: |value: &#tiled::PropertyValue| -> ::std::result::Result<::std::boxed::Box<dyn ::bevy::reflect::Reflect>, ::std::string::String> {
                    #from_property_impl
                },
            }
        }
    };

    // Generate FromTiledProperty trait implementation
    let from_tiled_property_match_arms: Vec<_> = analysis
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = &variant.name;

            match &variant.fields {
                None => {
                    // Unit variant
                    quote! {
                        #variant_name => ::std::option::Option::Some(Self::#variant_ident),
                    }
                }
                Some(VariantFields::Named(named_fields)) => {
                    // Struct variant
                    let field_extractions: Vec<_> = named_fields
                        .iter()
                        .map(|field| {
                            let field_ident = &field.ident;
                            let field_name = field_ident.to_string();
                            let field_type = &field.ty;

                            quote! {
                                let #field_ident: #field_type = properties
                                    .get(#field_name)
                                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))?;
                            }
                        })
                        .collect();

                    let field_names: Vec<_> = named_fields.iter().map(|f| &f.ident).collect();

                    quote! {
                        #variant_name => {
                            #(#field_extractions)*
                            ::std::option::Option::Some(Self::#variant_ident {
                                #(#field_names),*
                            })
                        },
                    }
                }
                Some(VariantFields::Unnamed(unnamed_fields)) => {
                    // Tuple variant
                    let field_extractions: Vec<_> = unnamed_fields
                        .iter()
                        .map(|field| {
                            let index = field.index;
                            let field_name = index.to_string();
                            let field_type = &field.ty;
                            let field_var = format_ident!("field_{}", index);

                            quote! {
                                let #field_var: #field_type = properties
                                    .get(#field_name)
                                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))?;
                            }
                        })
                        .collect();

                    let field_vars: Vec<_> = (0..unnamed_fields.len())
                        .map(|i| format_ident!("field_{}", i))
                        .collect();

                    quote! {
                        #variant_name => {
                            #(#field_extractions)*
                            ::std::option::Option::Some(Self::#variant_ident(
                                #(#field_vars),*
                            ))
                        },
                    }
                }
            }
        })
        .collect();

    let expanded = quote! {
        #variant_metadata_arrays

        // Implement FromTiledProperty for the enum
        impl #properties::FromTiledProperty for #enum_name {
            fn from_property(value: &#tiled::PropertyValue) -> ::std::option::Option<Self> {
                match value {
                    #tiled::PropertyValue::ClassValue { properties, .. } => {
                        // Extract :variant discriminant field
                        let variant_name = properties
                            .get(":variant")
                            .and_then(|v| match v {
                                #tiled::PropertyValue::StringValue(s) => ::std::option::Option::Some(s.as_str()),
                                _ => ::std::option::Option::None,
                            })?;

                        // Match on variant name and construct
                        match variant_name {
                            #(#from_tiled_property_match_arms)*
                            _ => ::std::option::Option::None,
                        }
                    }
                    _ => ::std::option::Option::None,
                }
            }
        }

        #inventory_submission
    };

    Ok(expanded.into())
}

/// Generate static field metadata arrays for all variants
fn generate_variant_metadata_arrays(
    enum_name: &syn::Ident,
    variants: &[VariantAnalysis],
    paths: &CratePaths,
) -> syn::Result<proc_macro2::TokenStream> {
    let properties = &paths.properties;
    let mut arrays = Vec::new();

    for variant in variants {
        if let Some(fields) = &variant.fields {
            let array_name = format_ident!(
                "__TILED_VARIANT_{}_{}",
                enum_name.to_string().to_uppercase(),
                variant.name.to_uppercase()
            );

            let field_entries = match fields {
                VariantFields::Named(named_fields) => named_fields
                    .iter()
                    .map(|field| {
                        let field_name = field.ident.to_string();
                        let tiled_type = map_rust_type_to_tiled(&field.ty, paths);
                        let default_value = generate_type_default(&field.ty, paths)?;

                        Ok(quote! {
                            #properties::TiledFieldInfo {
                                name: #field_name,
                                tiled_type: #tiled_type,
                                default_value: #default_value,
                            }
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()?,
                VariantFields::Unnamed(unnamed_fields) => unnamed_fields
                    .iter()
                    .map(|field| {
                        let field_name = field.index.to_string();
                        let tiled_type = map_rust_type_to_tiled(&field.ty, paths);
                        let default_value = generate_type_default(&field.ty, paths)?;

                        Ok(quote! {
                            #properties::TiledFieldInfo {
                                name: #field_name,
                                tiled_type: #tiled_type,
                                default_value: #default_value,
                            }
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()?,
            };

            arrays.push(quote! {
                #[doc(hidden)]
                static #array_name: &[#properties::TiledFieldInfo] = &[
                    #(#field_entries),*
                ];
            });
        }
    }

    Ok(quote! {
        #(#arrays)*
    })
}

/// Generate `TiledVariantInfo` static array
fn generate_variant_info_array(
    enum_name: &syn::Ident,
    variants: &[VariantAnalysis],
    paths: &CratePaths,
) -> syn::Result<proc_macro2::TokenStream> {
    let properties = &paths.properties;
    let _array_name = format_ident!(
        "__TILED_ENUM_VARIANTS_{}",
        enum_name.to_string().to_uppercase()
    );

    let variant_entries: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.name;
            let is_default = variant.is_default;

            let variant_kind = match &variant.fields {
                None => quote! {
                    #properties::TiledVariantKind::Unit
                },
                Some(VariantFields::Named(_)) => {
                    let fields_array_name = format_ident!(
                        "__TILED_VARIANT_{}_{}",
                        enum_name.to_string().to_uppercase(),
                        variant.name.to_uppercase()
                    );
                    quote! {
                        #properties::TiledVariantKind::Struct {
                            fields: #fields_array_name
                        }
                    }
                }
                Some(VariantFields::Unnamed(_)) => {
                    let fields_array_name = format_ident!(
                        "__TILED_VARIANT_{}_{}",
                        enum_name.to_string().to_uppercase(),
                        variant.name.to_uppercase()
                    );
                    quote! {
                        #properties::TiledVariantKind::Tuple {
                            fields: #fields_array_name
                        }
                    }
                }
            };

            quote! {
                #properties::TiledVariantInfo {
                    name: #variant_name,
                    kind: #variant_kind,
                    is_default: #is_default,
                }
            }
        })
        .collect();

    Ok(quote! { &[#(#variant_entries),*] })
}

/// Generate `FromTiledProperty` implementation for complex enum
fn generate_complex_from_property_impl(
    enum_name: &syn::Ident,
    tiled_name: &str,
    variants: &[VariantAnalysis],
    paths: &CratePaths,
) -> syn::Result<proc_macro2::TokenStream> {
    let properties = &paths.properties;
    let tiled = &paths.tiled;
    // Generate match arms for each variant
    let variant_match_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = &variant.name;

            match &variant.fields {
                None => {
                    // Unit variant
                    Ok(quote! {
                        #variant_name => ::std::result::Result::Ok(
                            ::std::boxed::Box::new(#enum_name::#variant_ident)
                        ),
                    })
                }
                Some(VariantFields::Named(named_fields)) => {
                    // Struct variant
                    let field_extractions: Vec<_> = named_fields
                        .iter()
                        .map(|field| {
                            let field_ident = &field.ident;
                            let field_name = field_ident.to_string();
                            let field_type = &field.ty;

                            quote! {
                                let #field_ident: #field_type = properties
                                    .get(#field_name)
                                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))
                                    .ok_or_else(|| ::std::format!(
                                        "Missing or invalid field '{}' for variant '{}'",
                                        #field_name,
                                        #variant_name
                                    ))?;
                            }
                        })
                        .collect();

                    let field_names: Vec<_> = named_fields.iter().map(|f| &f.ident).collect();

                    Ok(quote! {
                        #variant_name => {
                            #(#field_extractions)*
                            ::std::result::Result::Ok(::std::boxed::Box::new(#enum_name::#variant_ident {
                                #(#field_names),*
                            }))
                        },
                    })
                }
                Some(VariantFields::Unnamed(unnamed_fields)) => {
                    // Tuple variant
                    let field_extractions: Vec<_> = unnamed_fields
                        .iter()
                        .map(|field| {
                            let index = field.index;
                            let field_name = index.to_string();
                            let field_type = &field.ty;
                            let field_var = format_ident!("field_{}", index);

                            quote! {
                                let #field_var: #field_type = properties
                                    .get(#field_name)
                                    .and_then(|v| <#field_type as #properties::FromTiledProperty>::from_property(v))
                                    .ok_or_else(|| ::std::format!(
                                        "Missing or invalid field '{}' for variant '{}'",
                                        #field_name,
                                        #variant_name
                                    ))?;
                            }
                        })
                        .collect();

                    let field_vars: Vec<_> = (0..unnamed_fields.len())
                        .map(|i| format_ident!("field_{}", i))
                        .collect();

                    Ok(quote! {
                        #variant_name => {
                            #(#field_extractions)*
                            ::std::result::Result::Ok(::std::boxed::Box::new(#enum_name::#variant_ident(
                                #(#field_vars),*
                            )))
                        },
                    })
                }
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        match value {
            #tiled::PropertyValue::ClassValue { properties, .. } => {
                // Extract :variant discriminant field
                let variant_name = properties
                    .get(":variant")
                    .and_then(|v| match v {
                        #tiled::PropertyValue::StringValue(s) => ::std::option::Option::Some(s.as_str()),
                        _ => ::std::option::Option::None,
                    })
                    .ok_or_else(|| ::std::string::String::from(
                        "Missing or invalid ':variant' field in ClassValue"
                    ))?;

                // Match on variant name and construct
                match variant_name {
                    #(#variant_match_arms)*
                    _ => ::std::result::Result::Err(::std::format!(
                        "Unknown variant '{}' for enum '{}'",
                        variant_name,
                        #tiled_name
                    )),
                }
            }
            _ => ::std::result::Result::Err(::std::format!(
                "Expected ClassValue for complex enum '{}', got {:?}",
                #tiled_name,
                value
            )),
        }
    })
}

/// Parse #[tiled(name = "...")] attribute from struct
fn parse_tiled_name_attr(attrs: &[syn::Attribute]) -> syn::Result<String> {
    for attr in attrs {
        if !attr.path().is_ident("tiled") {
            continue;
        }

        let meta = &attr.meta;
        if let Meta::List(list) = meta {
            let nested: MetaNameValue = syn::parse2(list.tokens.clone())?;
            if nested.path.is_ident("name")
                && let syn::Expr::Lit(expr_lit) = &nested.value
                && let Lit::Str(lit_str) = &expr_lit.lit
            {
                return Ok(lit_str.value());
            }
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "TiledClass requires #[tiled(name = \"...\")] attribute",
    ))
}

/// Check if field has #[tiled(skip)] attribute
fn has_skip_attr(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("tiled") {
            continue;
        }

        if let Meta::List(list) = &attr.meta
            && let Ok(path) = syn::parse2::<syn::Path>(list.tokens.clone())
            && path.is_ident("skip")
        {
            return true;
        }
    }
    false
}

/// Parse #[tiled(default = ...)] attribute from field
fn parse_default_attr(attrs: &[syn::Attribute]) -> syn::Result<Option<proc_macro2::TokenStream>> {
    for attr in attrs {
        if !attr.path().is_ident("tiled") {
            continue;
        }

        if let Meta::List(list) = &attr.meta
            && let Ok(nested) = syn::parse2::<MetaNameValue>(list.tokens.clone())
            && nested.path.is_ident("default")
        {
            let value = &nested.value;
            return Ok(Some(quote! { #value }));
        }
    }
    Ok(None)
}

/// Extract inner type T from Option<T>, returns None if not an Option
fn extract_option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        // Extract the T from Option<T>
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Extract full type path from a `TypePath`.
///
/// For common types like glam vectors, infers the full module path.
/// For other types, returns just the type name.
fn extract_full_type_path(type_path: &syn::TypePath) -> String {
    let segments: Vec<String> = type_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();

    // If there's already a module path, use it
    if segments.len() > 1 {
        return segments.join("::");
    }

    // For simple types without module path, infer common ones
    let type_name = segments.last().unwrap();
    match type_name.as_str() {
        // Glam vector types - all variants use the same glam:: prefix
        "Vec2" | "Vec3" | "Vec4" | "IVec2" | "IVec3" | "IVec4" | "UVec2" | "UVec3" | "UVec4"
        | "DVec2" | "DVec3" | "DVec4" => format!("glam::{}", type_name),
        // For other types, just use the type name
        other => other.to_string(),
    }
}

/// Extract just the type name from a `TypePath` (last segment).
fn extract_type_name(type_path: &syn::TypePath) -> String {
    type_path
        .path
        .segments
        .last()
        .map(|seg| seg.ident.to_string())
        .unwrap_or_default()
}

/// Check if a type is `Handle<T>`.
fn is_handle_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Handle";
        }
    }
    false
}

/// Map Rust type to Tiled property type.
///
/// Returns a `TiledTypeKind` token stream for use in macro expansion.
fn map_rust_type_to_tiled(ty: &Type, paths: &CratePaths) -> proc_macro2::TokenStream {
    let properties = &paths.properties;
    // For Option<T>, unwrap to get the inner type
    let actual_type = extract_option_inner_type(ty).unwrap_or(ty);

    // Check for Handle<T> - these become File types (asset paths)
    if is_handle_type(actual_type) {
        return quote! { #properties::TiledTypeKind::File };
    }

    if let Type::Path(type_path) = actual_type {
        let type_name = extract_type_name(type_path);

        // Check if it's a primitive type
        match type_name.as_str() {
            "bool" => return quote! { #properties::TiledTypeKind::Bool },
            "i32" | "i64" | "i16" | "i8" | "u32" | "u64" | "u16" | "u8" | "usize" | "isize" => {
                return quote! { #properties::TiledTypeKind::Int };
            }
            "f32" | "f64" => return quote! { #properties::TiledTypeKind::Float },
            "String" | "str" => {
                return quote! { #properties::TiledTypeKind::String };
            }
            "Color" => return quote! { #properties::TiledTypeKind::Color },
            _ => {
                // Not a primitive - it's a referenced type (Vec2, custom types, etc.)
                let full_path = extract_full_type_path(type_path);
                return quote! {
                    #properties::TiledTypeKind::Class {
                        property_type: #full_path
                    }
                };
            }
        }
    }

    // Fallback for complex types
    quote! { #properties::TiledTypeKind::String }
}

/// Generate `TiledDefaultValue` expression for a field
fn generate_default_value_expr(
    ty: &Type,
    default_attr: &Option<proc_macro2::TokenStream>,
    paths: &CratePaths,
) -> syn::Result<proc_macro2::TokenStream> {
    // Get the actual type (unwrap Option if needed)
    let actual_type = extract_option_inner_type(ty).unwrap_or(ty);

    // If there's a #[tiled(default = ...)] attribute, use it
    if let Some(default_tokens) = default_attr {
        return generate_default_from_tokens(actual_type, default_tokens, paths);
    }

    // Otherwise generate a sensible default based on type
    generate_type_default(actual_type, paths)
}

/// Generate `TiledDefaultValue` from explicit default attribute
fn generate_default_from_tokens(
    ty: &Type,
    tokens: &proc_macro2::TokenStream,
    paths: &CratePaths,
) -> syn::Result<proc_macro2::TokenStream> {
    let properties = &paths.properties;
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();

        return Ok(match type_name.as_str() {
            "bool" => quote! {
                #properties::TiledDefaultValue::Bool(#tokens)
            },
            "i32" | "i64" | "i16" | "i8" | "u32" | "u64" | "u16" | "u8" => quote! {
                #properties::TiledDefaultValue::Int(#tokens as i32)
            },
            "f32" | "f64" => quote! {
                #properties::TiledDefaultValue::Float(#tokens as f32)
            },
            "Color" => {
                // Color defaults need special handling
                quote! {
                    #properties::TiledDefaultValue::Color { r: 255, g: 255, b: 255, a: 255 }
                }
            }
            _ => quote! {
                #properties::TiledDefaultValue::String("")
            },
        });
    }

    Ok(quote! {
        #properties::TiledDefaultValue::String("")
    })
}

/// Generate default `TiledDefaultValue` based on type alone
fn generate_type_default(ty: &Type, paths: &CratePaths) -> syn::Result<proc_macro2::TokenStream> {
    let properties = &paths.properties;
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();

        return Ok(match type_name.as_str() {
            "bool" => quote! {
                #properties::TiledDefaultValue::Bool(false)
            },
            "i32" | "i64" | "i16" | "i8" | "u32" | "u64" | "u16" | "u8" | "isize" | "usize" => {
                quote! {
                    #properties::TiledDefaultValue::Int(0)
                }
            }
            "f32" | "f64" => quote! {
                #properties::TiledDefaultValue::Float(0.0)
            },
            "Color" => quote! {
                #properties::TiledDefaultValue::Color { r: 255, g: 255, b: 255, a: 255 }
            },
            _ => quote! {
                #properties::TiledDefaultValue::String("")
            },
        });
    }

    Ok(quote! {
        #properties::TiledDefaultValue::String("")
    })
}
