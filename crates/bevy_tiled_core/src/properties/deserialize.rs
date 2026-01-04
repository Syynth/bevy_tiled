//! Property deserialization helpers.
//!
//! Provides the `FromTiledProperty` trait for converting Tiled `PropertyValue`
//! to Rust types.

use bevy::prelude::*;
use tiled::PropertyValue;

/// Trait for types that can be deserialized from Tiled properties.
///
/// This trait is automatically used by the `#[derive(TiledClass)]` macro to
/// convert property values to component fields.
///
/// # Example
///
/// ```ignore
/// use tiled::PropertyValue;
/// use bevy_tiled_core::properties::FromTiledProperty;
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
