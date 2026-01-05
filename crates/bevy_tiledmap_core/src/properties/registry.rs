//! Type registry for `TiledClass` components.
//!
//! Uses the inventory crate for compile-time registration of types marked with
//! `#[derive(TiledClass)]`.

use bevy::{asset::AssetServer, prelude::*};
use std::any::TypeId;
use std::collections::HashMap;
use std::string::String;
use tiled::{Properties, PropertyValue};

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
    /// File type (asset path for `Handle<T>` fields)
    ///
    /// When deserialized, this triggers asset loading via AssetServer.
    File,
    /// Class type (custom type with properties)
    ///
    /// The `property_type` field contains the full type path (e.g., "`glam::Vec2`", "`game::Door`")
    Class { property_type: &'static str },
    /// Enum type (unit-variant enums for dropdowns)
    ///
    /// The `property_type` field contains the full type path, and `variants` contains all variant names
    Enum {
        property_type: &'static str,
        variants: &'static [&'static str],
    },
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

/// Kind of enum variant (unit, struct, or tuple).
#[derive(Debug, Clone)]
pub enum TiledVariantKind {
    /// Unit variant (e.g., `None`, `North`)
    Unit,
    /// Struct variant with named fields (e.g., `Projectile { speed: f32, damage: i32 }`)
    Struct { fields: &'static [TiledFieldInfo] },
    /// Tuple variant with positional fields (e.g., `Dash(Vec2, f32)`)
    ///
    /// Field names are integers: "0", "1", "2", etc.
    Tuple { fields: &'static [TiledFieldInfo] },
}

/// Information about a single variant in an enum.
#[derive(Debug, Clone)]
pub struct TiledVariantInfo {
    /// Variant name (e.g., "North", "Projectile", "Dash")
    pub name: &'static str,

    /// Variant kind and associated field data
    pub kind: TiledVariantKind,

    /// Whether this variant has the `#[default]` attribute
    pub is_default: bool,
}

/// Kind of enum (simple unit-variant or complex with struct/tuple variants).
#[derive(Debug, Clone)]
pub enum TiledEnumKind {
    /// Simple enum with only unit variants (e.g., `Direction { North, South, East, West }`)
    ///
    /// Exported as Tiled's native enum type with dropdown UI.
    Simple {
        /// List of variant names
        variants: &'static [&'static str],
        /// Function to deserialize a string variant name into this enum type
        from_string: fn(&str) -> Result<Box<dyn Reflect>, String>,
    },

    /// Complex enum with struct and/or tuple variants (e.g., `Attack { None, Melee { damage: i32 } }`)
    ///
    /// Exported as Tiled class type with `:variant` discriminant field.
    Complex {
        /// Information about each variant
        variant_info: &'static [TiledVariantInfo],
    },
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

    /// Function to deserialize Tiled properties into this component type.
    ///
    /// The optional `AssetServer` parameter is required for loading `Handle<T>` fields.
    /// Pass `None` if the type has no asset handle fields.
    ///
    /// Returns a boxed reflected component or an error message.
    pub from_properties: fn(&Properties, Option<&AssetServer>) -> Result<Box<dyn Reflect>, String>,
}

// Collect all TiledClassInfo submissions at compile time
inventory::collect!(TiledClassInfo);

/// Information about a registered `TiledClass` enum type.
///
/// This struct is submitted via `inventory::submit!` by the `TiledClass` derive macro
/// for both simple (unit-variant) and complex (struct/tuple variant) enums.
/// Each registered enum provides:
/// - Its `TypeId` for reflection lookups
/// - A display name matching the Tiled custom enum name
/// - Enum kind (simple or complex) with variant information
/// - A deserialization function to convert property values to enum variants
pub struct TiledEnumInfo {
    /// The `TypeId` of the registered enum
    pub type_id: TypeId,

    /// The name used in Tiled custom properties (e.g., `"game::Direction"`, `"game::Attack"`)
    pub name: &'static str,

    /// Enum kind (simple or complex) with associated variant data
    pub kind: TiledEnumKind,

    /// Function to deserialize a property value into this enum type
    ///
    /// For simple enums, accepts `StringValue` with variant name.
    /// For complex enums, accepts `ClassValue` with `:variant` discriminant field.
    /// Returns a boxed reflected enum or an error message.
    pub from_property: fn(&PropertyValue) -> Result<Box<dyn Reflect>, String>,
}

// Collect all TiledEnumInfo submissions at compile time
inventory::collect!(TiledEnumInfo);

impl TiledEnumInfo {
    /// Get all variant names for this enum.
    ///
    /// Returns a slice of variant names in the order they're defined.
    pub fn variant_names(&self) -> Vec<&'static str> {
        match &self.kind {
            TiledEnumKind::Simple { variants, .. } => variants.to_vec(),
            TiledEnumKind::Complex { variant_info } => {
                variant_info.iter().map(|v| v.name).collect()
            }
        }
    }

    /// Check if this is a simple (unit-variant only) enum.
    pub fn is_simple(&self) -> bool {
        matches!(self.kind, TiledEnumKind::Simple { .. })
    }

    /// Check if this is a complex (struct/tuple variant) enum.
    pub fn is_complex(&self) -> bool {
        matches!(self.kind, TiledEnumKind::Complex { .. })
    }

    /// Get variant information by name (for complex enums).
    ///
    /// Returns `None` if this is a simple enum or if the variant doesn't exist.
    pub fn get_variant(&self, name: &str) -> Option<&TiledVariantInfo> {
        match &self.kind {
            TiledEnumKind::Simple { .. } => None,
            TiledEnumKind::Complex { variant_info } => {
                variant_info.iter().find(|v| v.name == name)
            }
        }
    }

    /// Get the default variant name if one exists.
    ///
    /// Returns `None` if no variant has the `#[default]` attribute.
    pub fn default_variant_name(&self) -> Option<&'static str> {
        match &self.kind {
            TiledEnumKind::Simple { .. } => None,
            TiledEnumKind::Complex { variant_info } => {
                variant_info.iter().find(|v| v.is_default).map(|v| v.name)
            }
        }
    }

    /// Get the variant info slice for complex enums.
    ///
    /// Returns `None` if this is a simple enum.
    pub fn variant_info(&self) -> Option<&[TiledVariantInfo]> {
        match &self.kind {
            TiledEnumKind::Simple { .. } => None,
            TiledEnumKind::Complex { variant_info } => Some(variant_info),
        }
    }

    /// Get the `from_string` function for simple enums.
    ///
    /// Returns `None` if this is a complex enum.
    pub fn from_string_fn(&self) -> Option<fn(&str) -> Result<Box<dyn Reflect>, String>> {
        match &self.kind {
            TiledEnumKind::Simple { from_string, .. } => Some(*from_string),
            TiledEnumKind::Complex { .. } => None,
        }
    }
}

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
    enums_by_name: HashMap<String, &'static TiledEnumInfo>,
}

impl TiledClassRegistry {
    /// Build the registry from all inventory submissions.
    ///
    /// This should be called once during plugin initialization.
    pub fn build() -> Self {
        let mut by_name = HashMap::new();
        let mut enums_by_name = HashMap::new();

        for info in inventory::iter::<TiledClassInfo> {
            by_name.insert(info.name.to_string(), info);
        }

        for info in inventory::iter::<TiledEnumInfo> {
            enums_by_name.insert(info.name.to_string(), info);
        }

        info!(
            "TiledClassRegistry built with {} registered types and {} enums",
            by_name.len(),
            enums_by_name.len()
        );

        Self {
            by_name,
            enums_by_name,
        }
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

    /// Get enum information by Tiled enum name.
    ///
    /// # Arguments
    ///
    /// * `name` - The enum name as it appears in Tiled properties (e.g., `"game::Direction"`)
    ///
    /// # Returns
    ///
    /// `Some(&TiledEnumInfo)` if an enum with this name was registered, `None` otherwise.
    pub fn get_enum(&self, name: &str) -> Option<&'static TiledEnumInfo> {
        self.enums_by_name.get(name).copied()
    }

    /// Iterate all registered enum names.
    pub fn enum_names(&self) -> impl Iterator<Item = &str> {
        self.enums_by_name.keys().map(String::as_str)
    }

    /// Iterate all registered enum info.
    pub fn iter_enums(&self) -> impl Iterator<Item = &'static TiledEnumInfo> + '_ {
        self.enums_by_name.values().copied()
    }

    /// Get the number of registered enums.
    pub fn enum_len(&self) -> usize {
        self.enums_by_name.len()
    }
}
