//! Tile layer spawning.

use bevy::prelude::*;
use tiled::LayerType;

use crate::components::tile::{TileInstance, TileLayerData};
use crate::systems::SpawnContext;

/// Build `TileLayerData` component from a tile layer.
///
/// Pre-processes all tiles: looks up tilesets by index, extracts flip flags.
///
/// # Arguments
///
/// * `layer` - The tile layer from the map asset
/// * `context` - Spawn context for tileset lookup
///
/// # Returns
///
/// `TileLayerData` component ready to attach to the layer entity
pub fn build_tile_layer_data(
    layer: &tiled::Layer,
    context: &SpawnContext,
) -> Option<TileLayerData> {
    // Only process tile layers
    let LayerType::Tiles(tile_layer) = layer.layer_type() else {
        return None;
    };

    let width = tile_layer.width().unwrap_or(0);
    let height = tile_layer.height().unwrap_or(0);

    let mut tile_data = TileLayerData::empty(width, height);

    // Iterate tiles and pre-process each one
    for y in 0..height {
        for x in 0..width {
            if let Some(tile) = tile_layer.get_tile(x as i32, y as i32) {
                // Get local tile ID directly from the tile (already resolved by tiled crate)
                let tile_id = tile.id();

                // Get tileset by index (matches our HashMap key)
                let tileset_index = tile.tileset_index();
                let Some(tileset_ref) = context.get_tileset_by_index(tileset_index as u32) else {
                    warn!(
                        "Tile at ({}, {}) references tileset index {} which doesn't exist",
                        x, y, tileset_index
                    );
                    continue;
                };

                // Get flip flags directly from the tile
                let flipped_h = tile.flip_h;
                let flipped_v = tile.flip_v;
                let flipped_d = tile.flip_d;

                // Create tile instance with local tile ID
                let tile_instance = TileInstance {
                    gid: tile_id, // Store local ID (we don't need GID anymore)
                    tileset_handle: tileset_ref.handle.clone(),
                    tile_id,
                    flipped_h,
                    flipped_v,
                    flipped_d,
                };

                tile_data.set(x, y, Some(tile_instance));
            }
        }
    }

    Some(tile_data)
}
