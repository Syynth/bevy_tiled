//! Observer for tile layer spawning events.

use bevy::prelude::*;
use bevy_tiled_assets::prelude::TiledTilesetAsset;
use bevy_tiled_core::components::tile::TileLayerData;
use bevy_tiled_core::events::TileLayerSpawned;

use super::tilemap_builder::TilemapBuilder;

/// Observer that renders tile layers when spawned by Layer 2.
///
/// This is the main entry point for tile layer rendering. When Layer 2 spawns
/// a tile layer entity with `TileLayerData`, this observer:
/// 1. Reads the pre-processed tile data
/// 2. Groups tiles by tileset
/// 3. Creates `bevy_ecs_tilemap` structures
/// 4. Spawns tilemap entities as children
pub fn on_tile_layer_spawned(
    trigger: On<TileLayerSpawned>,
    layer_query: Query<&TileLayerData>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok(tile_data) = layer_query.get(event.entity) else {
        warn!(
            "TileLayerSpawned event for entity {:?} but no TileLayerData component found",
            event.entity
        );
        return;
    };

    info!(
        "Rendering tile layer entity {:?} with {}x{} tiles",
        event.entity, tile_data.width, tile_data.height
    );

    // Build tilemap structures from tile data
    TilemapBuilder::build(&mut commands, event.entity, tile_data, &tileset_assets);
}
