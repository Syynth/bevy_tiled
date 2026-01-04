//! Basic example demonstrating map spawning and entity querying.
//!
//! This example shows:
//! - Spawning a `TiledMap` entity with an asset handle
//! - Automatic layer spawning by `process_loaded_maps()` system
//! - Querying layer entities and their types
//! - Accessing pre-processed tile data from `TileLayerData`
//! - Relationship traversal between maps and layers

use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tiledmap_assets::prelude::*;
use bevy_tiledmap_core::components::LayersInMap;
use bevy_tiledmap_core::components::tile::TileLayerData;
use bevy_tiledmap_core::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TiledmapAssetsPlugin,
            TiledmapCorePlugin::default(),
        ))
        // Add EguiPlugin before WorldInspectorPlugin
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, (setup_camera, spawn_map))
        .add_systems(Update, inspect_map)
        .run();
}

/// Setup a 2D camera for viewing
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn a map entity with a `TiledMap` component.
///
/// The map entity is spawned immediately with the asset handle.
/// The `process_loaded_maps()` system (from `TiledmapCorePlugin`) automatically:
/// 1. Detects the new `TiledMap` entity
/// 2. Waits for the asset to finish loading
/// 3. Spawns the layer hierarchy when ready
///
/// This is the recommended pattern - no manual asset polling needed!
fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("simple_map.tmx");

    commands.spawn((
        TiledMap { handle: map_handle },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Simple Map"),
    ));

    info!("Map entity spawned - layers will appear automatically when asset loads");
}

/// Inspect the spawned map and layer entities once they're ready.
///
/// This demonstrates:
/// - Querying for `TiledMap` entities
/// - Using `LayersInMap` to traverse from map to layers
/// - Querying layer types and components
/// - Accessing `TileLayerData` to inspect pre-processed tiles
fn inspect_map(
    map_query: Query<(Entity, &TiledMap, &LayersInMap, &Name)>,
    layer_query: Query<(Entity, &TiledLayer, &LayerId, &TiledLayerMapOf, &Name)>,
    tile_layer_query: Query<&TileLayerData>,
    mut inspected: Local<bool>,
) {
    // Only inspect once
    if *inspected {
        return;
    }

    // Try to get the map entity - return early if not found
    let Ok((map_entity, tiled_map, layers_in_map, map_name)) = map_query.single() else {
        return; // Map not spawned yet
    };

    // Wait until layers are spawned (process_loaded_maps adds LayersInMap)
    if layers_in_map.0.is_empty() {
        return; // Asset still loading, layers not spawned yet
    }

    *inspected = true;

    // At this point, assets are loaded and layers are spawned
    info!("=== Map: {} (Entity: {:?}) ===", map_name, map_entity);
    info!("  Asset Handle: {:?}", tiled_map.handle);
    info!("  Number of layers: {}", layers_in_map.0.len());

    // Iterate through all layers in this map
    for (i, &layer_entity) in layers_in_map.0.iter().enumerate() {
        if let Ok((entity, layer_type, layer_id, map_of, layer_name)) =
            layer_query.get(layer_entity)
        {
            info!("  Layer {}: {} (Entity: {:?})", i, layer_name, entity);
            info!("    Type: {:?}", layer_type);
            info!("    Layer ID: {}", layer_id.0);
            info!("    Parent Map: {:?}", map_of.0);

            // If this is a tile layer, inspect the tile data
            if let Ok(tile_data) = tile_layer_query.get(layer_entity) {
                info!("    --- Tile Layer Data ---");
                info!("    Dimensions: {}x{}", tile_data.width, tile_data.height);

                // Count non-empty tiles
                let tile_count: usize = tile_data.tiles.iter().filter(|t| t.is_some()).count();
                info!(
                    "    Tiles: {} / {}",
                    tile_count,
                    tile_data.width * tile_data.height
                );

                // Show first few tiles as example
                info!("    First 5 tiles:");
                for (x, y, tile) in tile_data.iter_tiles().take(5) {
                    info!(
                        "      ({}, {}): GID={}, TileID={}, Tileset={:?}, Flips=(H:{}, V:{}, D:{})",
                        x,
                        y,
                        tile.gid,
                        tile.tile_id,
                        tile.tileset_handle,
                        tile.flipped_h,
                        tile.flipped_v,
                        tile.flipped_d
                    );
                }
            }
        }
    }

    // Don't exit automatically - let the user explore with the inspector
    info!("Inspection complete - use the inspector to explore the entity hierarchy");
    info!("Press ESC to close the window");
}
