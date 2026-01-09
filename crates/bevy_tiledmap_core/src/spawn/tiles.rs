//! Tile layer spawning.

use std::panic;

use bevy::prelude::*;
use tiled::{LayerType, TileLayer};

use crate::components::tile::{TileInstance, TileLayerData};
use crate::systems::SpawnContext;

/// Build `TileLayerData` component from a tile layer.
///
/// Pre-processes all tiles: looks up tilesets by index, extracts flip flags.
/// Handles both finite (bounded) and infinite (chunk-based) tile layers.
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

    match tile_layer {
        TileLayer::Finite(finite_layer) => build_finite_tile_layer_data(finite_layer, context),
        TileLayer::Infinite(infinite_layer) => {
            build_infinite_tile_layer_data(infinite_layer, context)
        }
    }
}

/// Build tile layer data for finite (bounded) tile layers.
fn build_finite_tile_layer_data(
    tile_layer: tiled::FiniteTileLayer,
    context: &SpawnContext,
) -> Option<TileLayerData> {
    let width = tile_layer.width();
    let height = tile_layer.height();

    // Safety check: ensure dimensions are valid
    if width == 0 || height == 0 {
        return None;
    }

    let mut tile_data = TileLayerData::empty(width, height);

    // Iterate tiles and pre-process each one
    for y in 0..height {
        for x in 0..width {
            // Use catch_unwind to handle potential panics from the tiled crate
            // when layer dimensions don't match internal data array size
            let tile_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                tile_layer.get_tile(x as i32, y as i32)
            }));

            let tile = match tile_result {
                Ok(Some(t)) => t,
                Ok(None) => continue,
                Err(_) => {
                    // Panic occurred - layer data is likely malformed, skip remaining tiles
                    warn!(
                        "Tile layer has malformed data at ({}, {}), skipping remaining tiles",
                        x, y
                    );
                    return Some(tile_data);
                }
            };

            if let Some(tile_instance) = create_tile_instance(&tile, x, y, context) {
                tile_data.set(x, y, Some(tile_instance));
            }
        }
    }

    Some(tile_data)
}

/// Build tile layer data for infinite (chunk-based) tile layers.
///
/// Infinite maps store tiles in 16x16 chunks at arbitrary positions (including negative).
/// This function:
/// 1. Uses pre-calculated `tilemap_size` from the map asset
/// 2. Iterates all chunks and converts chunk-local coords to global tile coords
/// 3. Offsets coordinates so negative chunks map to positive tile indices
fn build_infinite_tile_layer_data(
    infinite_layer: tiled::InfiniteTileLayer,
    context: &SpawnContext,
) -> Option<TileLayerData> {
    // Get pre-calculated dimensions from map asset
    let width = context.map_asset.tilemap_size.x;
    let height = context.map_asset.tilemap_size.y;

    // Get the topleft chunk to calculate offset
    let (min_chunk_x, min_chunk_y) = context.map_asset.topleft_chunk;

    // Chunk dimensions (16x16 by default in tiled crate)
    let chunk_width = tiled::ChunkData::WIDTH;
    let chunk_height = tiled::ChunkData::HEIGHT;

    let mut tile_data = TileLayerData::empty(width, height);

    // Iterate all chunks in this layer
    for ((chunk_x, chunk_y), _chunk) in infinite_layer.chunks() {
        // Calculate the tile offset for this chunk
        // Shift by min_chunk to normalize negative chunks to positive indices
        // Use checked subtraction to catch any unexpected chunk positions
        let Some(rel_chunk_x) = chunk_x.checked_sub(min_chunk_x).and_then(|v| u32::try_from(v).ok()) else {
            warn!("Chunk at ({}, {}) is outside expected bounds (min: {}, {})", chunk_x, chunk_y, min_chunk_x, min_chunk_y);
            continue;
        };
        let Some(rel_chunk_y) = chunk_y.checked_sub(min_chunk_y).and_then(|v| u32::try_from(v).ok()) else {
            warn!("Chunk at ({}, {}) is outside expected bounds (min: {}, {})", chunk_x, chunk_y, min_chunk_x, min_chunk_y);
            continue;
        };
        let chunk_offset_x = rel_chunk_x * chunk_width;
        let chunk_offset_y = rel_chunk_y * chunk_height;

        // Iterate tiles within the chunk (0..16 for both x and y)
        for local_y in 0..chunk_height {
            for local_x in 0..chunk_width {
                // Get tile using global tile coordinates
                let global_tile_x = chunk_x * chunk_width as i32 + local_x as i32;
                let global_tile_y = chunk_y * chunk_height as i32 + local_y as i32;

                if let Some(tile) = infinite_layer.get_tile(global_tile_x, global_tile_y) {
                    // Calculate normalized tile position in our grid
                    let tile_x = chunk_offset_x + local_x;
                    let tile_y = chunk_offset_y + local_y;

                    if let Some(tile_instance) =
                        create_tile_instance(&tile, tile_x, tile_y, context)
                    {
                        tile_data.set(tile_x, tile_y, Some(tile_instance));
                    }
                }
            }
        }
    }

    Some(tile_data)
}

/// Create a `TileInstance` from a `LayerTile`, handling tileset lookup and flip flags.
fn create_tile_instance(
    tile: &tiled::LayerTile,
    x: u32,
    y: u32,
    context: &SpawnContext,
) -> Option<TileInstance> {
    let tile_id = tile.id();
    let tileset_index = tile.tileset_index();

    let Some(tileset_ref) = context.get_tileset_by_index(tileset_index as u32) else {
        warn!(
            "Tile at ({}, {}) references tileset index {} which doesn't exist",
            x, y, tileset_index
        );
        return None;
    };

    Some(TileInstance {
        gid: tile_id, // Store local ID (we don't need GID anymore)
        tileset_handle: tileset_ref.handle.clone(),
        tile_id,
        flipped_h: tile.flip_h,
        flipped_v: tile.flip_v,
        flipped_d: tile.flip_d,
    })
}
