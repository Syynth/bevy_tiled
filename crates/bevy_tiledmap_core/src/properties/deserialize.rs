//! Property deserialization helpers.
//!
//! Provides the `FromTiledProperty` trait for converting Tiled `PropertyValue`
//! to Rust types.

use bevy::app::App;
use bevy::prelude::*;
use bevy::reflect::{ReflectMut, TypeInfo, TypeRegistry, TypeRegistration};
use tiled::{Properties, PropertyValue};

use super::registry::TiledClassRegistry;

/// Trait for types that can be deserialized from Tiled properties.
///
/// This trait is automatically used by the `#[derive(TiledClass)]` macro to
/// convert property values to component fields.
///
/// # Example
///
/// ```ignore
/// use tiled::PropertyValue;
/// use bevy_tiledmap_core::properties::FromTiledProperty;
///
/// let prop = PropertyValue::BoolValue(true);
/// let value: bool = bool::from_property(&prop).unwrap();
/// assert_eq!(value, true);
/// ```
pub trait FromTiledProperty: Sized {
    /// Attempt to convert a Tiled property value to this type.
    ///
    /// Returns `Some(value)` if conversion succeeds, `None` otherwise.
    fn from_property(value: &PropertyValue) -> Option<Self>;
}

// Primitive type implementations

impl FromTiledProperty for bool {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::BoolValue(b) => Some(*b),
            _ => None,
        }
    }
}

impl FromTiledProperty for i32 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::IntValue(i) => Some(*i),
            _ => None,
        }
    }
}

impl FromTiledProperty for u32 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::IntValue(i) if *i >= 0 => Some(*i as u32),
            _ => None,
        }
    }
}

impl FromTiledProperty for f32 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::FloatValue(f) => Some(*f),
            PropertyValue::IntValue(i) => Some(*i as f32),
            _ => None,
        }
    }
}

impl FromTiledProperty for f64 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::FloatValue(f) => Some(*f as f64),
            PropertyValue::IntValue(i) => Some(*i as f64),
            _ => None,
        }
    }
}

impl FromTiledProperty for String {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::StringValue(s) => Some(s.clone()),
            _ => None,
        }
    }
}

// Bevy type implementations

impl FromTiledProperty for Color {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::ColorValue(color) => {
                // tiled::Color has alpha, red, green, blue fields (u8)
                let r = color.red as f32 / 255.0;
                let g = color.green as f32 / 255.0;
                let b = color.blue as f32 / 255.0;
                let a = color.alpha as f32 / 255.0;

                Some(Color::srgba(r, g, b, a))
            }
            _ => None,
        }
    }
}

impl FromTiledProperty for Vec2 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::StringValue(s) => {
                // Parse "x,y" format
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 2 {
                    let x = parts[0].trim().parse::<f32>().ok()?;
                    let y = parts[1].trim().parse::<f32>().ok()?;
                    Some(Vec2::new(x, y))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl FromTiledProperty for Vec3 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::StringValue(s) => {
                // Parse "x,y,z" format
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 3 {
                    let x = parts[0].trim().parse::<f32>().ok()?;
                    let y = parts[1].trim().parse::<f32>().ok()?;
                    let z = parts[2].trim().parse::<f32>().ok()?;
                    Some(Vec3::new(x, y, z))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// Option<T> implementation
impl<T: FromTiledProperty> FromTiledProperty for Option<T> {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        // For Option<T>, we return Some(Some(T)) if conversion succeeds,
        // Some(None) if the property is explicitly null/empty,
        // None only if the type conversion fails
        match T::from_property(value) {
            Some(inner) => Some(Some(inner)),
            None => {
                // Check if this is an explicitly empty/null value
                match value {
                    PropertyValue::StringValue(s) if s.is_empty() => Some(None),
                    _ => None,
                }
            }
        }
    }
}

// ============================================================================
// Hybrid Class Deserialization (TiledClass + Reflection)
// ============================================================================

/// Error type for class deserialization.
#[derive(Debug, Clone)]
pub enum DeserializeError {
    /// Type not found in either `TiledClass` registry or reflection
    UnknownType(String),
    /// Type found but is not a struct
    NotAStruct(String),
    /// Type found but doesn't have Default implementation
    NoDefault(String),
    /// Field not found on type
    FieldNotFound(String),
    /// Property value has wrong type
    TypeError(String),
    /// Enum variant not found
    UnknownVariant(String),
    /// Unsupported variant kind (e.g., complex variants not yet supported)
    UnsupportedVariantKind(String),
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::UnknownType(name) => {
                write!(
                    f,
                    "Type '{}' not found in TiledClassRegistry or AppTypeRegistry. \
                     Add #[derive(TiledClass)] or #[derive(Reflect, Default)]",
                    name
                )
            }
            DeserializeError::NotAStruct(name) => {
                write!(f, "Type '{}' is not a struct", name)
            }
            DeserializeError::NoDefault(name) => {
                write!(
                    f,
                    "Type '{}' doesn't implement Default (required for reflection)",
                    name
                )
            }
            DeserializeError::FieldNotFound(name) => {
                write!(f, "Field '{}' not found", name)
            }
            DeserializeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            DeserializeError::UnknownVariant(name) => {
                write!(f, "Unknown enum variant '{}'", name)
            }
            DeserializeError::UnsupportedVariantKind(msg) => {
                write!(f, "Unsupported variant kind: {}", msg)
            }
        }
    }
}

impl std::error::Error for DeserializeError {}

/// Deserialize a class-typed property using hybrid lookup.
///
/// This function uses the following strategy:
/// 1. Try `TiledClass` registry first (for manually registered types)
/// 2. Fall back to Bevy reflection (for types with Reflect + Default)
/// 3. Return error if type not found
///
/// # Arguments
///
/// * `property_type` - The full type path (e.g., "`glam::Vec2`", "`game::Door`")
/// * `properties` - The property values to deserialize
/// * `app` - The Bevy App (for accessing registries)
///
/// # Returns
///
/// A boxed reflected value on success, or a `DeserializeError`
pub fn deserialize_class(
    property_type: &str,
    properties: &Properties,
    app: &App,
) -> Result<Box<dyn Reflect>, DeserializeError> {
    // 1. Try TiledClass registry first
    let tiled_registry = app.world().resource::<TiledClassRegistry>();
    if let Some(tiled_class) = tiled_registry.get(property_type) {
        return (tiled_class.from_properties)(properties)
            .map_err(DeserializeError::TypeError);
    }

    // 2. Fall back to Bevy reflection
    let app_type_registry = app.world().resource::<AppTypeRegistry>();
    let registry = app_type_registry.read();

    if let Some(reflect_type) = registry.get_with_type_path(property_type) {
        return deserialize_reflected(reflect_type, properties, &registry, app);
    }

    // 3. Type not found
    Err(DeserializeError::UnknownType(property_type.to_string()))
}

/// Deserialize an enum from a string variant name using hybrid lookup.
///
/// This function uses the following strategy:
/// 1. Try `TiledClass` enum registry first (for types with `#[derive(TiledClass)]`)
/// 2. Fall back to Bevy reflection (for types with `#[derive(Reflect)]`)
/// 3. Return error if type not found or variant invalid
///
/// # Arguments
///
/// * `enum_name` - The full type path (e.g., "`game::Direction`")
/// * `variant_str` - The variant name (e.g., "`North`")
/// * `app` - The Bevy App (for accessing registries)
///
/// # Returns
///
/// A boxed reflected enum value on success, or a `DeserializeError`
pub fn deserialize_enum_from_string(
    enum_name: &str,
    variant_str: &str,
    app: &App,
) -> Result<Box<dyn Reflect>, DeserializeError> {
    // 1. Try TiledClass enum registry first
    let tiled_registry = app.world().resource::<TiledClassRegistry>();
    if let Some(enum_info) = tiled_registry.get_enum(enum_name) {
        // For simple enums, use the from_string function
        if let Some(from_string) = enum_info.from_string_fn() {
            return from_string(variant_str).map_err(DeserializeError::TypeError);
        }
        // For complex enums, this function shouldn't be called (use ClassValue instead)
        return Err(DeserializeError::TypeError(format!(
            "Enum '{}' is a complex enum and cannot be deserialized from a string. Use ClassValue with :variant field.",
            enum_name
        )));
    }

    // 2. Fall back to Bevy reflection
    let app_type_registry = app.world().resource::<AppTypeRegistry>();
    let registry = app_type_registry.read();

    if let Some(reflect_type) = registry.get_with_type_path(enum_name)
        && let TypeInfo::Enum(enum_info) = reflect_type.type_info()
    {
        return deserialize_enum_via_reflection(enum_info, variant_str);
    }

    // 3. Type not found
    Err(DeserializeError::UnknownType(enum_name.to_string()))
}

/// Deserialize an enum using Bevy's reflection system.
///
/// This is a helper function for reflection-based enum deserialization.
/// Currently only supports unit variants via `TiledClass` registry.
///
/// Note: Full reflection-based enum deserialization is not yet implemented
/// because `DynamicEnum` construction requires additional trait bounds.
/// Use `#[derive(TiledClass)]` on your enum types for proper deserialization.
fn deserialize_enum_via_reflection(
    enum_info: &bevy::reflect::EnumInfo,
    variant_name: &str,
) -> Result<Box<dyn Reflect>, DeserializeError> {
    use bevy::reflect::VariantInfo;

    // Validate that the variant exists
    let variant_index = enum_info
        .index_of(variant_name)
        .ok_or_else(|| DeserializeError::UnknownVariant(variant_name.to_string()))?;

    let variant_info = enum_info.variant_at(variant_index).unwrap();

    // For now, we only support TiledClass enums (handled above in deserialize_enum_from_string)
    // Full reflection-based enum construction requires more complex setup
    match variant_info {
        VariantInfo::Unit(_) => Err(DeserializeError::TypeError(format!(
            "Enum '{}' found via reflection but not in TiledClass registry. \
             Add #[derive(TiledClass)] to enable deserialization.",
            enum_info.type_path()
        ))),
        VariantInfo::Struct(_) => Err(DeserializeError::UnsupportedVariantKind(
            "Struct variants not yet supported".to_string(),
        )),
        VariantInfo::Tuple(_) => Err(DeserializeError::UnsupportedVariantKind(
            "Tuple variants not yet supported".to_string(),
        )),
    }
}

/// Deserialize a type using Bevy's reflection system.
fn deserialize_reflected(
    reflect_type: &TypeRegistration,
    properties: &Properties,
    _registry: &TypeRegistry,
    app: &App,
) -> Result<Box<dyn Reflect>, DeserializeError> {
    let type_info = reflect_type.type_info();

    let TypeInfo::Struct(struct_info) = type_info else {
        return Err(DeserializeError::NotAStruct(
            type_info.type_path().to_string(),
        ));
    };

    // Create default instance
    let reflect_default = reflect_type
        .data::<ReflectDefault>()
        .ok_or_else(|| DeserializeError::NoDefault(type_info.type_path().to_string()))?;

    let mut value = reflect_default.default();

    // Set fields from properties
    for (prop_name, prop_value) in properties {
        if struct_info.field(prop_name).is_none() {
            warn!(
                "Unknown field '{}' on type '{}', skipping",
                prop_name,
                type_info.type_path()
            );
            continue;
        }

        // Deserialize the property value
        let field_value = deserialize_property_value(prop_value, app)?;

        // Apply to the field by name using ReflectMut
        match value.reflect_mut() {
            ReflectMut::Struct(struct_mut) => {
                if let Some(field_mut) = struct_mut.field_mut(prop_name) {
                    field_mut.apply(&*field_value);
                } else {
                    return Err(DeserializeError::FieldNotFound(prop_name.clone()));
                }
            }
            _ => {
                return Err(DeserializeError::TypeError(format!(
                    "Type '{}' is not a struct",
                    type_info.type_path()
                )));
            }
        }
    }

    Ok(value)
}

/// Deserialize a `PropertyValue` to a reflected value.
fn deserialize_property_value(
    value: &PropertyValue,
    app: &App,
) -> Result<Box<dyn Reflect>, DeserializeError> {
    match value {
        PropertyValue::BoolValue(b) => Ok(Box::new(*b)),
        PropertyValue::IntValue(i) => Ok(Box::new(*i)),
        PropertyValue::FloatValue(f) => Ok(Box::new(*f)),
        PropertyValue::StringValue(s) => Ok(Box::new(s.clone())),
        PropertyValue::ColorValue(c) => {
            // Convert tiled::Color to bevy::Color
            let r = c.red as f32 / 255.0;
            let g = c.green as f32 / 255.0;
            let b = c.blue as f32 / 255.0;
            let a = c.alpha as f32 / 255.0;
            Ok(Box::new(Color::srgba(r, g, b, a)))
        }
        PropertyValue::ClassValue {
            property_type,
            properties,
        } => {
            // Recursively deserialize nested class
            deserialize_class(property_type, properties, app)
        }
        PropertyValue::FileValue(path) => Ok(Box::new(path.clone())),
        PropertyValue::ObjectValue(id) => Ok(Box::new(*id)),
    }
}
