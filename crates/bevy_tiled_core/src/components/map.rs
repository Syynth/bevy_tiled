//! Map components and relationship components.

use bevy::prelude::*;
use bevy_tiled_assets::prelude::TiledMapAsset;

/// Root component for a Tiled map.
///
/// Spawn an entity with this component to trigger map loading and entity hierarchy creation.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiled_core::TiledMap;
/// fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn(TiledMap {
///         handle: asset_server.load("maps/level1.tmx"),
///     });
/// }
/// ```
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct TiledMap {
    /// Handle to the loaded `TiledMapAsset`.
    pub handle: Handle<TiledMapAsset>,
}

// ===== RELATIONSHIP COMPONENTS =====
//
// These components implement bidirectional relationships using Bevy's relationship system.
// Note: You may need to add relationship attributes depending on your Bevy version/setup.

/// Relationship: Layer → Map
///
/// Points from a layer entity to its parent map entity.
/// Paired with `LayersInMap` for bidirectional traversal.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TiledLayerMapOf(pub Entity);

/// Relationship target: Map → Layers
///
/// Auto-synced list of layer entities belonging to this map.
/// Paired with `TiledLayerMapOf` for bidirectional traversal.
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct LayersInMap(pub Vec<Entity>);

/// Relationship: Object → Map
///
/// Points from an object entity to its parent map entity.
/// Paired with `ObjectsInMap` for bidirectional traversal.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TiledObjectMapOf(pub Entity);

/// Relationship target: Map → Objects
///
/// Auto-synced list of object entities belonging to this map.
/// Paired with `TiledObjectMapOf` for bidirectional traversal.
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct ObjectsInMap(pub Vec<Entity>);

/// Relationship: Map → World (for future world support)
///
/// Points from a map entity to its parent world entity.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TiledWorldOf(pub Entity);

/// Relationship target: World → Maps (for future world support)
///
/// Auto-synced list of map entities belonging to this world.
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct MapsInWorld(pub Vec<Entity>);
