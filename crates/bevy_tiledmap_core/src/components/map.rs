//! Map and world components and relationship components.

use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::{TiledMapAsset, TiledWorldAsset};

/// Marker component for scene roots (both maps and worlds).
///
/// This component is automatically added to both `TiledMap` and `TiledWorld` entities,
/// allowing you to query for any Tiled scene root without distinguishing between them.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_core::prelude::TiledSceneRoot;
/// fn find_scene_root(query: Query<Entity, With<TiledSceneRoot>>) {
///     for entity in &query {
///         // Works for both maps and worlds
///         println!("Scene root: {:?}", entity);
///     }
/// }
/// ```
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TiledSceneRoot;

/// Root component for a Tiled world.
///
/// Spawn an entity with this component to trigger world loading and entity hierarchy creation.
/// A world contains multiple maps positioned in a larger game world.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_core::prelude::TiledWorld;
/// fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn(TiledWorld {
///         handle: asset_server.load("worlds/overworld.world"),
///     });
/// }
/// ```
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct TiledWorld {
    /// Handle to the loaded `TiledWorldAsset`.
    pub handle: Handle<TiledWorldAsset>,
}

/// Root component for a Tiled map.
///
/// Spawn an entity with this component to trigger map loading and entity hierarchy creation.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_core::prelude::TiledMap;
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

// ===== GEOMETRY COMPONENT =====

/// World-space geometry information for a Tiled map.
///
/// Provides the world-space boundary and coordinate conversion utilities.
/// Attached to map entities during spawning.
///
/// # Coordinate System
///
/// - Origin (0, 0) is at the bottom-left corner of the map
/// - X increases rightward (positive)
/// - Y increases upward (positive) - standard Bevy convention
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_core::components::map::MapGeometry;
/// fn check_player_in_bounds(
///     player_query: Query<&Transform, With<Player>>,
///     map_query: Query<&MapGeometry>,
/// ) {
///     let player_pos = player_query.single().translation.truncate();
///     let map_geometry = map_query.single();
///
///     if map_geometry.bounds.contains(player_pos) {
///         // Player is within map bounds
///     }
/// }
/// # #[derive(Component)] struct Player;
/// ```
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MapGeometry {
    /// Map dimensions in tiles
    pub size: UVec2,
    /// Tile dimensions in pixels/world units
    pub tile_size: Vec2,
    /// World-space bounding rectangle of the map.
    /// - `min` is at (0, 0) - bottom-left corner
    /// - `max` is at `(width * tile_width, height * tile_height)` - top-right corner
    ///
    /// Use this directly for `.intersect()`, `.contains()`, etc.
    pub bounds: Rect,
}

impl MapGeometry {
    /// Create a new `MapGeometry` from map dimensions.
    pub fn new(width: u32, height: u32, tile_width: f32, tile_height: f32) -> Self {
        Self {
            size: UVec2::new(width, height),
            tile_size: Vec2::new(tile_width, tile_height),
            bounds: Rect {
                min: Vec2::ZERO,
                max: Vec2::new(width as f32 * tile_width, height as f32 * tile_height),
            },
        }
    }

    /// Convert a tile grid coordinate to world-space position (center of tile).
    ///
    /// Uses Tiled's coordinate system for input (y=0 is top row).
    /// Returns the center position of the tile in Bevy world space (local to the map entity).
    ///
    /// Returns `None` if the tile coordinate is out of bounds.
    pub fn tile_to_world(&self, tile_x: u32, tile_y: u32) -> Option<Vec2> {
        if tile_x >= self.size.x || tile_y >= self.size.y {
            return None;
        }
        // Flip Y: Tiled y=0 is top, Bevy y=0 is bottom
        let flipped_y = self.size.y - 1 - tile_y;
        Some(Vec2::new(
            (tile_x as f32 + 0.5) * self.tile_size.x,
            (flipped_y as f32 + 0.5) * self.tile_size.y,
        ))
    }

    /// Convert a world-space position to tile grid coordinate.
    ///
    /// Returns Tiled's coordinate system (y=0 is top row).
    /// Returns `None` if the position is outside the map bounds.
    pub fn world_to_tile(&self, world_pos: Vec2) -> Option<UVec2> {
        if !self.bounds.contains(world_pos) {
            return None;
        }
        let tile_x = (world_pos.x / self.tile_size.x) as u32;
        // Flip Y back: Bevy y at bottom → Tiled y at top
        let bevy_tile_y = (world_pos.y / self.tile_size.y) as u32;
        let tile_y = self.size.y.saturating_sub(1).saturating_sub(bevy_tile_y);
        Some(UVec2::new(
            tile_x.min(self.size.x.saturating_sub(1)),
            tile_y.min(self.size.y.saturating_sub(1)),
        ))
    }

    /// Get the world-space rectangle for a specific tile.
    ///
    /// Uses Tiled's coordinate system for input (y=0 is top row).
    /// Returns `None` if the tile coordinate is out of bounds.
    pub fn tile_rect(&self, tile_x: u32, tile_y: u32) -> Option<Rect> {
        if tile_x >= self.size.x || tile_y >= self.size.y {
            return None;
        }
        let flipped_y = self.size.y - 1 - tile_y;
        let min = Vec2::new(
            tile_x as f32 * self.tile_size.x,
            flipped_y as f32 * self.tile_size.y,
        );
        Some(Rect {
            min,
            max: min + self.tile_size,
        })
    }
}
