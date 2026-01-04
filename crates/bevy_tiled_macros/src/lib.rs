//! Procedural macros for `bevy_tiled`.
//!
//! This crate provides the `TiledClass` derive macro for automatic component
//! registration and property deserialization.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, MetaNameValue, Type, parse_macro_input};

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
/// use bevy_tiled_macros::TiledClass;
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

fn derive_tiled_class_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Parse #[tiled(name = "...")] attribute on struct
    let tiled_name = parse_tiled_name_attr(&input.attrs)?;

    // Only support structs with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "TiledClass only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "TiledClass can only be derived for structs",
            ));
        }
    };

    // Generate field deserialization code and metadata
    let mut field_inits = Vec::new();
    let mut field_metadata = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check for #[tiled(skip)]
        if has_skip_attr(&field.attrs) {
            // Use Default::default() for skipped fields
            field_inits.push(quote! {
                #field_name: ::std::default::Default::default()
            });
            // Skipped fields don't appear in metadata
            continue;
        }

        // Check for #[tiled(default = ...)]
        let default_value = parse_default_attr(&field.attrs)?;

        // Generate field metadata for JSON export
        let field_name_str = field_name.to_string();
        let tiled_type = map_rust_type_to_tiled(field_type);
        let default_expr = generate_default_value_expr(field_type, &default_value)?;

        field_metadata.push(quote! {
            bevy_tiled_core::properties::TiledFieldInfo {
                name: #field_name_str,
                tiled_type: #tiled_type,
                default_value: #default_expr,
            }
        });

        // Generate property access code
        let field_init = if let Some(inner_type) = extract_option_inner_type(field_type) {
            // Optional fields: extract inner type T from Option<T>
            quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#inner_type as bevy_tiled_core::properties::FromTiledProperty>::from_property(v))
            }
        } else if let Some(default) = default_value {
            // Fields with defaults use the default if missing
            quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#field_type as bevy_tiled_core::properties::FromTiledProperty>::from_property(v))
                    .unwrap_or(#default)
            }
        } else {
            // Required fields error if missing
            quote! {
                #field_name: __properties.get(#field_name_str)
                    .and_then(|v| <#field_type as bevy_tiled_core::properties::FromTiledProperty>::from_property(v))
                    .ok_or_else(|| format!("Missing required property '{}'", #field_name_str))?
            }
        };

        field_inits.push(field_init);
    }

    // Generate static field metadata array (uppercase for lint compliance)
    let fields_array_name =
        quote::format_ident!("__TILED_FIELDS_{}", struct_name.to_string().to_uppercase());

    // Generate the complete implementation
    let expanded = quote! {
        // Static array of field metadata for JSON export
        #[doc(hidden)]
        static #fields_array_name: &[bevy_tiled_core::properties::TiledFieldInfo] = &[
            #(#field_metadata),*
        ];

        // Submit to inventory for compile-time registration
        ::inventory::submit! {
            bevy_tiled_core::properties::TiledClassInfo {
                type_id: ::std::any::TypeId::of::<#struct_name>(),
                name: #tiled_name,
                fields: #fields_array_name,
                from_properties: #struct_name::__tiled_from_properties,
            }
        }

        impl #struct_name {
            #[doc(hidden)]
            fn __tiled_from_properties(
                __properties: &::tiled::Properties,
            ) -> ::std::result::Result<::std::boxed::Box<dyn ::bevy::reflect::Reflect>, ::std::string::String> {
                let instance = Self {
                    #(#field_inits),*
                };

                Ok(::std::boxed::Box::new(instance))
            }
        }
    };

    Ok(expanded.into())
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

/// Map Rust type to Tiled property type.
///
/// Returns a `TiledTypeKind` token stream for use in macro expansion.
fn map_rust_type_to_tiled(ty: &Type) -> proc_macro2::TokenStream {
    // For Option<T>, unwrap to get the inner type
    let actual_type = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = actual_type {
        let type_name = extract_type_name(type_path);

        // Check if it's a primitive type
        match type_name.as_str() {
            "bool" => return quote! { bevy_tiled_core::properties::TiledTypeKind::Bool },
            "i32" | "i64" | "i16" | "i8" | "u32" | "u64" | "u16" | "u8" | "usize" | "isize" => {
                return quote! { bevy_tiled_core::properties::TiledTypeKind::Int };
            }
            "f32" | "f64" => return quote! { bevy_tiled_core::properties::TiledTypeKind::Float },
            "String" | "str" => {
                return quote! { bevy_tiled_core::properties::TiledTypeKind::String };
            }
            "Color" => return quote! { bevy_tiled_core::properties::TiledTypeKind::Color },
            _ => {
                // Not a primitive - it's a referenced type (Vec2, custom types, etc.)
                let full_path = extract_full_type_path(type_path);
                return quote! {
                    bevy_tiled_core::properties::TiledTypeKind::Class {
                        property_type: #full_path
                    }
                };
            }
        }
    }

    // Fallback for complex types
    quote! { bevy_tiled_core::properties::TiledTypeKind::String }
}

/// Generate `TiledDefaultValue` expression for a field
fn generate_default_value_expr(
    ty: &Type,
    default_attr: &Option<proc_macro2::TokenStream>,
) -> syn::Result<proc_macro2::TokenStream> {
    // Get the actual type (unwrap Option if needed)
    let actual_type = extract_option_inner_type(ty).unwrap_or(ty);

    // If there's a #[tiled(default = ...)] attribute, use it
    if let Some(default_tokens) = default_attr {
        return generate_default_from_tokens(actual_type, default_tokens);
    }

    // Otherwise generate a sensible default based on type
    generate_type_default(actual_type)
}

/// Generate `TiledDefaultValue` from explicit default attribute
fn generate_default_from_tokens(
    ty: &Type,
    tokens: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();

        return Ok(match type_name.as_str() {
            "bool" => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::Bool(#tokens)
            },
            "i32" | "i64" | "i16" | "i8" | "u32" | "u64" | "u16" | "u8" => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::Int(#tokens as i32)
            },
            "f32" | "f64" => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::Float(#tokens as f32)
            },
            "Color" => {
                // Color defaults need special handling
                quote! {
                    bevy_tiled_core::properties::TiledDefaultValue::Color { r: 255, g: 255, b: 255, a: 255 }
                }
            }
            _ => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::String("")
            },
        });
    }

    Ok(quote! {
        bevy_tiled_core::properties::TiledDefaultValue::String("")
    })
}

/// Generate default `TiledDefaultValue` based on type alone
fn generate_type_default(ty: &Type) -> syn::Result<proc_macro2::TokenStream> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();

        return Ok(match type_name.as_str() {
            "bool" => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::Bool(false)
            },
            "i32" | "i64" | "i16" | "i8" | "u32" | "u64" | "u16" | "u8" | "isize" | "usize" => {
                quote! {
                    bevy_tiled_core::properties::TiledDefaultValue::Int(0)
                }
            }
            "f32" | "f64" => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::Float(0.0)
            },
            "Color" => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::Color { r: 255, g: 255, b: 255, a: 255 }
            },
            _ => quote! {
                bevy_tiled_core::properties::TiledDefaultValue::String("")
            },
        });
    }

    Ok(quote! {
        bevy_tiled_core::properties::TiledDefaultValue::String("")
    })
}
