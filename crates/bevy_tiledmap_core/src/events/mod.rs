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

/// Fired when a map's entity hierarchy is fully spawned.
///
/// This event is triggered after all layers and objects have been created
/// for a map. Use this for initialization that requires the complete map
/// structure to be in place.
///
/// This is an `EntityEvent` that can be observed on the spawned entity.
///
/// # Example
///
/// ```ignore
/// commands.spawn(TiledMap { ... })
///     .observe(|trigger: On<MapSpawned>, mut readiness: ResMut<GameReadiness>| {
///         info!("Map fully loaded: {:?}", trigger.event().entity);
///         readiness.map_ready = true;
///     });
/// ```
#[derive(EntityEvent, Debug, Clone)]
pub struct MapSpawned {
    /// The map entity
    #[event_target]
    pub entity: Entity,
}

/// Fired when a world and all its maps are fully spawned.
///
/// This event is triggered after the world entity and all child map entities
/// have been fully processed. Use this for initialization that requires
/// the entire world structure to be in place.
///
/// This is an `EntityEvent` that can be observed on the spawned entity.
///
/// # Example
///
/// ```ignore
/// commands.spawn(TiledWorld { ... })
///     .observe(|trigger: On<WorldSpawned>, mut readiness: ResMut<GameReadiness>| {
///         info!("World fully loaded: {:?}", trigger.event().entity);
///         readiness.world_ready = true;
///     });
/// ```
#[derive(EntityEvent, Debug, Clone)]
pub struct WorldSpawned {
    /// The world entity
    #[event_target]
    pub entity: Entity,
}
