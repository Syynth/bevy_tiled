//! Type registry for `TiledClass` components.
//!
//! Uses the inventory crate for compile-time registration of types marked with
//! `#[derive(TiledClass)]`.

use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::string::String;
use tiled::Properties;

/// Information about a registered `TiledClass` type.
///
/// This struct is submitted via `inventory::submit!` by the `TiledClass` derive macro.
/// Each registered type provides:
/// - Its `TypeId` for reflection lookups
/// - A display name matching the Tiled custom class name
/// - A deserialization function to convert properties to a component
pub struct TiledClassInfo {
    /// The `TypeId` of the registered component
    pub type_id: TypeId,

    /// The name used in Tiled custom properties (e.g., `"game::Door"`)
    pub name: &'static str,

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

    /// Get the number of registered types.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}
