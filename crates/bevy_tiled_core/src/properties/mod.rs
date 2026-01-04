//! Property handling and component registration for Tiled custom properties.
//!
//! This module provides:
//! - Type registry for `#[derive(TiledClass)]` components
//! - JSON export for Tiled editor integration
//! - Property deserialization (Phase 2)
//! - Merged property data (Phase 4)

use bevy::prelude::*;

pub mod deserialize;
pub mod export;
pub mod registry;

pub use deserialize::FromTiledProperty;
pub use export::{
    build_export_data, export_types_to_json, TiledMemberExport, TiledTypeExport, TiledValueExport,
};
pub use registry::{TiledClassInfo, TiledClassRegistry, TiledDefaultValue, TiledFieldInfo};

/// Pre-merged properties stored as a component.
///
/// This component is automatically attached to objects and layers during spawning.
/// It contains the merged properties from templates (if applicable) and the object/layer itself.
///
/// # Use Cases
///
/// 1. **Layer 3 access to raw properties**: Physics/rendering plugins can read custom properties
/// 2. **Conditional logic**: Check properties to decide whether to attach other components
/// 3. **Data-driven behavior**: Use properties to configure gameplay systems
///
/// # Example
///
/// ```ignore
/// fn my_system(query: Query<(Entity, &MergedProperties, &TiledObject)>) {
///     for (entity, props, object) in query.iter() {
///         if let Some(damage) = props.get_i32("damage") {
///             // Use the damage value
///         }
///     }
/// }
/// ```
#[derive(Component, Debug, Clone /*, Reflect */)]
// #[reflect(Component)] // TODO: Reflect can't work on tiled::Properties
pub struct MergedProperties {
    properties: tiled::Properties,
}

impl MergedProperties {
    /// Create a new `MergedProperties` from a Properties map.
    pub fn new(properties: tiled::Properties) -> Self {
        Self { properties }
    }

    /// Get a property value by key.
    pub fn get(&self, key: &str) -> Option<&tiled::PropertyValue> {
        self.properties.get(key)
    }

    /// Get a boolean property value.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            tiled::PropertyValue::BoolValue(b) => Some(*b),
            _ => None,
        }
    }

    /// Get an integer property value.
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        match self.get(key)? {
            tiled::PropertyValue::IntValue(i) => Some(*i),
            _ => None,
        }
    }

    /// Get a float property value.
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        match self.get(key)? {
            tiled::PropertyValue::FloatValue(f) => Some(*f),
            tiled::PropertyValue::IntValue(i) => Some(*i as f32),
            _ => None,
        }
    }

    /// Get a string property value.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            tiled::PropertyValue::StringValue(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get a color property value.
    pub fn get_color(&self, key: &str) -> Option<tiled::Color> {
        match self.get(key)? {
            tiled::PropertyValue::ColorValue(c) => Some(*c),
            _ => None,
        }
    }

    /// Iterate all properties.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &tiled::PropertyValue)> {
        self.properties.iter()
    }

    /// Get the number of properties.
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Check if there are no properties.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}
