//! Event system for Layer 3 extension hooks.
//!
//! These events allow Layer 3 plugins (rendering, physics) to hook into the spawning
//! process and access property data for conditional logic and component attachment.

use bevy::prelude::*;
use tiled::Properties;

/// Fired when an object entity is spawned.
///
/// Layer 3 plugins can use this event to:
/// - Attach rendering components based on object properties
/// - Set up physics colliders based on object shape
/// - Initialize gameplay systems based on custom properties
///
/// # Example
///
/// ```ignore
/// fn handle_object_spawned(
///     mut events: EventReader<ObjectSpawned>,
///     mut commands: Commands,
/// ) {
///     for event in events.read() {
///         // Check if object should have physics
///         if let Some(PropertyValue::BoolValue(true)) = event.properties.get("has_physics") {
///             commands.entity(event.entity).insert(RigidBody::Dynamic);
///         }
///     }
/// }
/// ```
#[derive(Event, Debug, Clone)]
pub struct ObjectSpawned {
    /// The spawned object entity
    pub entity: Entity,
    /// The parent map entity
    pub map_entity: Entity,
    /// The object's ID from Tiled
    pub object_id: u32,
    /// Merged properties (template + object overrides)
    pub properties: Properties,
}

/// Fired when a tile layer is spawned.
///
/// Layer 3 plugins can use this event to:
/// - Set up tile rendering systems
/// - Generate collision meshes from tile data
/// - Initialize layer-specific effects (parallax, fog, etc.)
///
/// # Example
///
/// ```ignore
/// fn handle_tile_layer_spawned(
///     mut events: EventReader<TileLayerSpawned>,
///     mut commands: Commands,
/// ) {
///     for event in events.read() {
///         // Add parallax effect to background layers
///         if event.properties.get("parallax").is_some() {
///             commands.entity(event.entity).insert(ParallaxLayer::default());
///         }
///     }
/// }
/// ```
#[derive(Event, Debug, Clone)]
pub struct TileLayerSpawned {
    /// The spawned layer entity
    pub entity: Entity,
    /// The parent map entity
    pub map_entity: Entity,
    /// The layer's ID from Tiled
    pub layer_id: u32,
    /// Layer properties
    pub properties: Properties,
}

/// Fired when an object layer is spawned.
#[derive(Event, Debug, Clone)]
pub struct ObjectLayerSpawned {
    /// The spawned layer entity
    pub entity: Entity,
    /// The parent map entity
    pub map_entity: Entity,
    /// The layer's ID from Tiled
    pub layer_id: u32,
    /// Layer properties
    pub properties: Properties,
}

/// Fired when an image layer is spawned.
#[derive(Event, Debug, Clone)]
pub struct ImageLayerSpawned {
    /// The spawned layer entity
    pub entity: Entity,
    /// The parent map entity
    pub map_entity: Entity,
    /// The layer's ID from Tiled
    pub layer_id: u32,
    /// Layer properties
    pub properties: Properties,
}

/// Fired when a group layer is spawned.
#[derive(Event, Debug, Clone)]
pub struct GroupLayerSpawned {
    /// The spawned layer entity
    pub entity: Entity,
    /// The parent map entity
    pub map_entity: Entity,
    /// The layer's ID from Tiled
    pub layer_id: u32,
    /// Layer properties
    pub properties: Properties,
}
