//! Main reactive spawning system.

use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;
use bevy_tiled_assets::prelude::{TiledMapAsset, TiledTilesetAsset};

use crate::components::TiledMap;
use crate::spawn::spawn_map;
use crate::systems::SpawnContext;

/// Marker component to trigger map respawning.
///
/// Add this component to force the map to be respawned even if it hasn't changed.
#[derive(Component)]
pub struct RespawnTiledMap;

/// Reactive system that detects when `TiledMapAsset` loading completes and spawns entities.
///
/// Runs in `PreUpdate` before user systems.
///
/// # Triggers
///
/// - `Changed<TiledMap>` - When map handle is added or changed
/// - `With<RespawnTiledMap>` - When manual respawn is requested
///
/// # Use Cases
///
/// 1. **Initial spawn**: User creates entity with `TiledMap` component
/// 2. **Hot reload**: Asset changes, system detects and respawns
/// 3. **Dynamic loading**: Runtime map loading for procedural levels
pub fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    map_assets: Res<Assets<TiledMapAsset>>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
    mut map_query: Query<(Entity, &TiledMap), Or<(Changed<TiledMap>, With<RespawnTiledMap>)>>,
) {
    for (map_entity, tiled_map) in map_query.iter_mut() {
        // Check if all dependencies have finished loading
        let load_state = asset_server.get_recursive_dependency_load_state(&tiled_map.handle);

        if !matches!(load_state, Some(RecursiveDependencyLoadState::Loaded)) {
            // Still loading, skip for now
            continue;
        }

        // Get the map asset
        let Some(map_asset) = map_assets.get(&tiled_map.handle) else {
            // Asset handle loaded but asset not found
            continue;
        };

        // Create spawn context with asset references
        let context = SpawnContext::new(map_asset, &tileset_assets);

        // Spawn the map hierarchy
        spawn_map(&mut commands, map_entity, &context);

        // Remove RespawnTiledMap marker if present
        commands.entity(map_entity).remove::<RespawnTiledMap>();
    }
}
