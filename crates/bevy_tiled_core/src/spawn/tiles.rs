//! Tile layer spawning.

use tiled::LayerType;

use crate::components::tile::{TileInstance, TileLayerData};
use crate::systems::SpawnContext;

/// Build `TileLayerData` component from a tile layer.
///
/// Pre-processes all tiles: resolves GIDs, extracts flip flags, looks up tilesets.
///
/// # Arguments
///
/// * `layer` - The tile layer from the map asset
/// * `context` - Spawn context for GID resolution
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
                let gid = tile.id();

                // Skip empty tiles (GID 0)
                if gid == 0 {
                    continue;
                }

                // Resolve GID to tileset + local tile ID
                let Some((tileset_handle, tile_id)) = context.resolve_gid(gid) else {
                    continue;
                };

                // Extract flip flags
                let (flipped_h, flipped_v, flipped_d) = SpawnContext::extract_flip_flags(gid);

                // Create tile instance
                let tile_instance = TileInstance {
                    gid,
                    tileset_handle,
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
