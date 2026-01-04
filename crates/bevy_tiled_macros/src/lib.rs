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

    // Generate field deserialization code
    let mut field_inits = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check for #[tiled(skip)]
        if has_skip_attr(&field.attrs) {
            // Use Default::default() for skipped fields
            field_inits.push(quote! {
                #field_name: ::std::default::Default::default()
            });
            continue;
        }

        // Check for #[tiled(default = ...)]
        let default_value = parse_default_attr(&field.attrs)?;

        // Generate property access code
        let field_name_str = field_name.to_string();
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

    // Generate the complete implementation
    let expanded = quote! {
        // Submit to inventory for compile-time registration
        ::inventory::submit! {
            bevy_tiled_core::properties::TiledClassInfo {
                type_id: ::std::any::TypeId::of::<#struct_name>(),
                name: #tiled_name,
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
