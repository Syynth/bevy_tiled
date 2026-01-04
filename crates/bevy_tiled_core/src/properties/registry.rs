//! Type registry for `TiledClass` components.
//!
//! Uses the inventory crate for compile-time registration of types marked with
//! `#[derive(TiledClass)]`.

use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::string::String;
use tiled::Properties;

/// Tiled property type kind.
///
/// Represents the type system used by Tiled for custom properties.
/// For non-primitive types, use `Class` with a `property_type` string.
#[derive(Debug, Clone, PartialEq)]
pub enum TiledTypeKind {
    /// Boolean type (`true`/`false`)
    Bool,
    /// Integer type
    Int,
    /// Floating point type
    Float,
    /// String type
    String,
    /// Color type (#AARRGGBB format)
    Color,
    /// Class type (custom type with properties)
    ///
    /// The `property_type` field contains the full type path (e.g., "glam::Vec2", "game::Door")
    Class { property_type: &'static str },
}

/// Default value for a Tiled class field.
///
/// Represents the default value in a format that can be exported to Tiled's JSON.
#[derive(Debug, Clone)]
pub enum TiledDefaultValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(&'static str),
    Color { r: u8, g: u8, b: u8, a: u8 },
}

/// Information about a single field in a `TiledClass`.
///
/// Used for JSON export to provide autocomplete in Tiled editor.
#[derive(Debug, Clone)]
pub struct TiledFieldInfo {
    /// Field name
    pub name: &'static str,

    /// Tiled property type
    pub tiled_type: TiledTypeKind,

    /// Default value for this field
    pub default_value: TiledDefaultValue,
}

/// Information about a registered `TiledClass` type.
///
/// This struct is submitted via `inventory::submit!` by the `TiledClass` derive macro.
/// Each registered type provides:
/// - Its `TypeId` for reflection lookups
/// - A display name matching the Tiled custom class name
/// - Field metadata for JSON export
/// - A deserialization function to convert properties to a component
pub struct TiledClassInfo {
    /// The `TypeId` of the registered component
    pub type_id: TypeId,

    /// The name used in Tiled custom properties (e.g., `"game::Door"`)
    pub name: &'static str,

    /// Field definitions for JSON export
    pub fields: &'static [TiledFieldInfo],

    /// Function to deserialize Tiled properties into this component type
    /// Returns a boxed reflected component or an error message
    pub from_properties: fn(&Properties) -> Result<Box<dyn Reflect>, String>,
}

// Collect all TiledClassInfo submissions at compile time
inventory::collect!(TiledClassInfo);

/// Registry of all types with `#[derive(TiledClass)]`.
///
/// This resource is built at plugin startup by iterating all compile-time
/// registered types via the inventory crate.
///
/// # Example
///
/// ```ignore
/// fn my_system(registry: Res<TiledClassRegistry>) {
///     if let Some(info) = registry.get("game::Door") {
///         // Can deserialize Door components from properties
///     }
/// }
/// ```
#[derive(Resource)]
pub struct TiledClassRegistry {
    by_name: HashMap<String, &'static TiledClassInfo>,
}

impl TiledClassRegistry {
    /// Build the registry from all inventory submissions.
    ///
    /// This should be called once during plugin initialization.
    pub fn build() -> Self {
        let mut by_name = HashMap::new();

        for info in inventory::iter::<TiledClassInfo> {
            by_name.insert(info.name.to_string(), info);
        }

        info!(
            "TiledClassRegistry built with {} registered types",
            by_name.len()
        );

        Self { by_name }
    }

    /// Get type information by Tiled class name.
    ///
    /// # Arguments
    ///
    /// * `name` - The class name as it appears in Tiled properties (e.g., `"game::Door"`)
    ///
    /// # Returns
    ///
    /// `Some(&TiledClassInfo)` if a type with this name was registered, `None` otherwise.
    pub fn get(&self, name: &str) -> Option<&'static TiledClassInfo> {
        self.by_name.get(name).copied()
    }

    /// Iterate all registered type names.
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        self.by_name.keys().map(String::as_str)
    }

    /// Iterate all registered class info.
    pub fn iter(&self) -> impl Iterator<Item = &'static TiledClassInfo> + '_ {
        self.by_name.values().copied()
    }

    /// Get the number of registered types.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}
