//! Tile collider generation from tileset collision shapes.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::TiledTilesetAsset;
use bevy_tiledmap_core::events::TileLayerSpawned;
use std::collections::{HashMap, HashSet};

use crate::config::{PhysicsConfig, TileColliderStrategy};
use crate::shapes;

/// Observer that generates physics colliders for tile layers.
///
/// When a tile layer is spawned, this observer:
/// 1. Checks if tile colliders are enabled in `PhysicsConfig`
/// 2. Extracts tiles with collision shapes from the tileset
/// 3. Generates colliders based on the configured strategy:
///    - `PerTileEntity`: Individual child entities per tile
///    - `CompoundMerged`: Optimized compound with rectangle merging (recommended)
///    - `CompoundChunked`: Chunked compounds for large maps
///
/// # Rectangle Merging Optimization
///
/// The `CompoundMerged` strategy uses a greedy algorithm to merge rectangular
/// tiles into larger shapes, dramatically reducing collider count (5-100x reduction).
///
/// Algorithm:
/// 1. Group tiles by collision shape type
/// 2. For rectangular tiles, perform horizontal-then-vertical merging
/// 3. For custom shapes, add directly to compound
/// 4. Create single compound collider with all optimized shapes
pub fn on_tile_layer_spawned(
    trigger: On<TileLayerSpawned>,
    layer_query: Query<&bevy_tiledmap_core::components::tile::TileLayerData>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    config: Res<PhysicsConfig>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Check if tile colliders are enabled
    if !config.enable_tile_colliders {
        return;
    }

    // Get the tile layer data
    let Ok(tile_data) = layer_query.get(event.entity) else {
        warn!("TileLayerSpawned event for entity without TileLayerData component");
        return;
    };

    // Generate colliders based on strategy
    match config.tile_collider_strategy {
        TileColliderStrategy::Disabled => {
            // No-op
        }

        TileColliderStrategy::PerTileEntity => {
            info!(
                "Generating per-tile entity colliders for layer {} (NOT IMPLEMENTED YET)",
                event.layer_id
            );
            // TODO: Implement in future iteration
        }

        TileColliderStrategy::CompoundMerged => {
            generate_merged_compound_collider(
                event.entity,
                tile_data,
                &tileset_assets,
                &mut commands,
            );
        }

        TileColliderStrategy::CompoundChunked => {
            info!(
                "Generating chunked compound colliders for layer {} (NOT IMPLEMENTED YET)",
                event.layer_id
            );
            // TODO: Implement in future iteration
        }
    }
}

/// Generate optimized compound collider with rectangle merging.
///
/// This is the recommended strategy for static terrain. It merges contiguous
/// rectangular tiles into larger shapes, reducing collider count by 5-100x.
///
/// # Algorithm
///
/// 1. Extract all tiles with collision shapes from the layer
/// 2. Group tiles by collision shape type (rectangle vs custom)
/// 3. For rectangular tiles:
///    - Sort by position (scanline order)
///    - Merge horizontally (extend right as far as possible)
///    - Merge vertically (extend strips downward)
/// 4. For custom shapes, add directly to compound
/// 5. Create compound collider on layer entity
fn generate_merged_compound_collider(
    layer_entity: Entity,
    tile_data: &bevy_tiledmap_core::components::tile::TileLayerData,
    tileset_assets: &Assets<TiledTilesetAsset>,
    commands: &mut Commands,
) {
    // Step 1: Collect tiles with collision shapes, grouped by tileset+shape
    let mut rectangular_tiles: HashMap<TileCollisionKey, Vec<(u32, u32)>> = HashMap::new();
    let mut custom_shapes: Vec<(Vec2, f32, Collider)> = Vec::new();

    // We need to know tile size for positioning. Extract it from the first tileset we encounter
    let mut tile_size = Vec2::new(16.0, 16.0); // Default fallback
    let map_height = tile_data.height;

    for (x, y, tile_instance) in tile_data.iter_tiles() {
        // Get the tileset for this tile
        let Some(tileset) = tileset_assets.get(&tile_instance.tileset_handle) else {
            continue;
        };

        // Update tile_size from tileset (assume all tilesets have same tile size)
        tile_size = Vec2::new(tileset.tile_size.x as f32, tileset.tile_size.y as f32);

        // Check if this tile has collision shapes
        if !shapes::tile_has_collision_shape(tileset, tile_instance.tile_id) {
            continue;
        }

        // Check if it's a simple rectangle (can be merged)
        if let Some((width, height)) = shapes::get_tile_rectangle_collision_size(tileset, tile_instance.tile_id) {
            // Rectangular collision - can be merged
            let key = TileCollisionKey {
                tileset_id: tile_instance.tileset_handle.id(),
                tile_id: tile_instance.tile_id,
                rect_size_bits: (width.to_bits(), height.to_bits()),
            };
            rectangular_tiles.entry(key).or_default().push((x, y));
        } else {
            // Custom shape - add individual shapes directly to avoid nested compounds
            let tile_shapes = shapes::get_tile_collision_shapes(tileset, tile_instance.tile_id);
            if !tile_shapes.is_empty() {
                // Calculate tile center position to match tilemap rendering
                // Use positive Y with Y-flip to match MapGeometry bounds
                let flipped_y = map_height - 1 - y;
                let tile_local_pos = Vec2::new(
                    (x as f32 + 0.5) * tile_size.x,
                    (flipped_y as f32 + 0.5) * tile_size.y,
                );

                // Add each shape with its offset relative to tile center
                for (shape_offset, rotation, collider) in tile_shapes {
                    let local_pos = tile_local_pos + shape_offset;
                    custom_shapes.push((local_pos, rotation, collider));
                }
            }
        }
    }

    // Step 2: Merge rectangular tiles into optimized strips
    let mut merged_colliders = Vec::new();
    let total_tiles_before = rectangular_tiles.values().map(Vec::len).sum::<usize>();

    for (_key, positions) in rectangular_tiles {
        let strips = merge_rectangular_tiles_into_strips(positions, tile_size, map_height);
        for (center, size) in strips {
            merged_colliders.push((center, 0.0, Collider::rectangle(size.x, size.y)));
        }
    }

    let rectangles_after = merged_colliders.len();

    // Step 3: Add custom shapes
    merged_colliders.extend(custom_shapes);

    // Step 4: Create compound collider on layer entity
    if !merged_colliders.is_empty() {
        let total_shapes = merged_colliders.len();

        commands.entity(layer_entity).insert((
            RigidBody::Static,
            Collider::compound(merged_colliders),
        ));

        info!(
            "Generated compound collider with {} shapes (merged {} rectangular tiles into {} rectangles, {} custom shapes)",
            total_shapes,
            total_tiles_before,
            rectangles_after,
            total_shapes - rectangles_after
        );
    } else {
        info!("No tiles with collision shapes found in layer");
    }
}

/// Key for grouping rectangular tiles that can be merged together.
///
/// Tiles can only be merged if they have identical collision shapes.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct TileCollisionKey {
    /// Tileset asset ID (tiles from different tilesets can't be merged)
    tileset_id: AssetId<TiledTilesetAsset>,
    /// Tile ID within the tileset (different tiles can't be merged)
    tile_id: u32,
    /// Rectangle size for collision (quantized to avoid float comparison issues)
    /// Stored as (`width_bits`, `height_bits`) for exact comparison
    rect_size_bits: (u32, u32),
}

/// Merge rectangular tiles into horizontal/vertical strips.
///
/// Uses a greedy horizontal-then-vertical approach:
/// 1. Sort positions by (y, x) for scanline processing
/// 2. For each unvisited position:
///    a. Extend horizontally as far as possible
///    b. Create a strip with that width
///    c. Extend that strip vertically (keeping width constant)
///    d. Mark all positions in the merged rectangle as visited
///
/// # Arguments
///
/// * `positions` - Grid positions of tiles to merge
/// * `tile_size` - Size of each tile in world units
/// * `map_height` - Height of the map in tiles (for Y-flip)
///
/// # Returns
///
/// Vector of (`center_position`, size) for each merged rectangle.
/// Uses positive Y with Y-flip to match MapGeometry bounds.
fn merge_rectangular_tiles_into_strips(
    positions: Vec<(u32, u32)>,
    tile_size: Vec2,
    map_height: u32,
) -> Vec<(Vec2, Vec2)> {
    let mut grid: HashSet<(u32, u32)> = positions.into_iter().collect();
    let mut strips = Vec::new();

    // Process tiles in scanline order
    let mut sorted_positions: Vec<_> = grid.iter().copied().collect();
    sorted_positions.sort_by_key(|&(x, y)| (y, x));

    for (start_x, start_y) in sorted_positions {
        if !grid.contains(&(start_x, start_y)) {
            continue; // Already merged
        }

        // Extend horizontally
        let mut width = 1;
        while grid.contains(&(start_x + width, start_y)) {
            width += 1;
        }

        // Try to extend vertically (keeping width constant)
        let mut height = 1;
        'vertical: loop {
            // Check if entire row of width exists below
            for dx in 0..width {
                if !grid.contains(&(start_x + dx, start_y + height)) {
                    break 'vertical;
                }
            }
            height += 1;
        }

        // Remove merged tiles from grid
        for dy in 0..height {
            for dx in 0..width {
                grid.remove(&(start_x + dx, start_y + dy));
            }
        }

        // Calculate center position and size
        // Use positive Y with Y-flip to match MapGeometry bounds
        let strip_width = width as f32 * tile_size.x;
        let strip_height = height as f32 * tile_size.y;
        let center_x = (start_x as f32 + width as f32 / 2.0) * tile_size.x;
        // Y-flip: start_y is top of strip in Tiled coords, convert to Bevy positive Y
        let flipped_y = map_height as f32 - start_y as f32 - height as f32 / 2.0;
        let center_y = flipped_y * tile_size.y;

        strips.push((
            Vec2::new(center_x, center_y),
            Vec2::new(strip_width, strip_height),
        ));
    }

    strips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_single_tile() {
        let positions = vec![(0, 0)];
        let tile_size = Vec2::new(16.0, 16.0);
        let map_height = 10; // 10 tiles tall
        let strips = merge_rectangular_tiles_into_strips(positions, tile_size, map_height);

        assert_eq!(strips.len(), 1);
        // Positive Y with Y-flip to match MapGeometry
        // Tile at (0,0) in 10-tile map: flipped_y = 10 - 0 - 0.5 = 9.5, center_y = 9.5 * 16 = 152
        assert_eq!(strips[0].0, Vec2::new(8.0, 152.0)); // Center
        assert_eq!(strips[0].1, Vec2::new(16.0, 16.0)); // Size
    }

    #[test]
    fn test_merge_horizontal_strip() {
        let positions = vec![(0, 0), (1, 0), (2, 0)];
        let tile_size = Vec2::new(16.0, 16.0);
        let map_height = 10;
        let strips = merge_rectangular_tiles_into_strips(positions, tile_size, map_height);

        assert_eq!(strips.len(), 1);
        // Positive Y with Y-flip
        // 3-tile strip at y=0: flipped_y = 10 - 0 - 0.5 = 9.5, center_y = 152
        assert_eq!(strips[0].0, Vec2::new(24.0, 152.0)); // Center of 3-wide strip
        assert_eq!(strips[0].1, Vec2::new(48.0, 16.0)); // 3 tiles wide
    }

    #[test]
    fn test_merge_rectangle() {
        // 2x2 square
        let positions = vec![(0, 0), (1, 0), (0, 1), (1, 1)];
        let tile_size = Vec2::new(16.0, 16.0);
        let map_height = 10;
        let strips = merge_rectangular_tiles_into_strips(positions, tile_size, map_height);

        assert_eq!(strips.len(), 1);
        // Positive Y with Y-flip
        // 2x2 block starting at (0,0): flipped_y = 10 - 0 - 1 = 9, center_y = 9 * 16 = 144
        assert_eq!(strips[0].0, Vec2::new(16.0, 144.0)); // Center of 2x2
        assert_eq!(strips[0].1, Vec2::new(32.0, 32.0)); // 2x2 tiles
    }

    #[test]
    fn test_merge_l_shape() {
        // L-shaped pattern - should create 2 rectangles
        let positions = vec![
            (0, 0), (1, 0), (2, 0), // Horizontal part
            (0, 1), (0, 2), // Vertical part
        ];
        let tile_size = Vec2::new(16.0, 16.0);
        let map_height = 10;
        let strips = merge_rectangular_tiles_into_strips(positions, tile_size, map_height);

        // Should merge into 2 rectangles (greedy algorithm)
        assert_eq!(strips.len(), 2);
    }
}
