//! Tile layer data components.
//!
//! Individual tiles are NOT spawned as entities. Tile data is stored in the
//! `TileLayerData` component attached to tile layer entities.

use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::TiledTilesetAsset;

/// Raw tile grid data attached to tile layer entities.
///
/// Layer 3 rendering plugins decide how to render this (`bevy_ecs_tilemap`, native tilemap, sprites, etc.).
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_core::components::tile::TileLayerData;
/// fn render_tile_layer(
///     layer_query: Query<&TileLayerData>,
/// ) {
///     for tile_data in layer_query.iter() {
///         for (x, y, tile) in tile_data.iter_tiles() {
///             // Layer 3 decides rendering: TileStorage, native tilemap, sprites, etc.
///             println!("Tile at ({}, {}): tileset {:?}, tile_id {}", x, y, tile.tileset_handle, tile.tile_id);
///         }
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TileLayerData {
    /// Map width in tiles
    pub width: u32,

    /// Map height in tiles
    pub height: u32,

    /// Flattened grid of tiles: index = y * width + x
    /// None = empty tile
    pub tiles: Vec<Option<TileInstance>>,
}

impl TileLayerData {
    /// Create an empty tile layer with the given dimensions.
    pub fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![None; (width * height) as usize],
        }
    }

    /// Get tile at position (returns None if out of bounds or empty).
    pub fn get(&self, x: u32, y: u32) -> Option<&TileInstance> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.get((y * self.width + x) as usize)?.as_ref()
    }

    /// Set tile at position.
    pub fn set(&mut self, x: u32, y: u32, tile: Option<TileInstance>) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if let Some(slot) = self.tiles.get_mut(index) {
                *slot = tile;
            }
        }
    }

    /// Iterate all non-empty tiles with their positions.
    ///
    /// Returns `(x, y, tile_instance)` tuples where x, y are grid coordinates
    /// in Tiled's coordinate system (Y-down, origin at top-left).
    pub fn iter_tiles(&self) -> impl Iterator<Item = (u32, u32, &TileInstance)> {
        self.tiles.iter().enumerate().filter_map(|(idx, tile)| {
            tile.as_ref().map(|t| {
                let x = (idx as u32) % self.width;
                let y = (idx as u32) / self.width;
                (x, y, t)
            })
        })
    }

    /// Convert a grid position to world position (center of tile).
    ///
    /// Handles the coordinate system conversion from Tiled (Y-down, top-left origin)
    /// to Bevy (Y-up, bottom-left origin).
    ///
    /// # Coordinate System
    ///
    /// - Origin (0, 0) is at the bottom-left corner of the map
    /// - X increases rightward (positive)
    /// - Y increases upward (positive)
    /// - Returns the center position of the tile
    ///
    /// # Arguments
    ///
    /// * `x` - Grid X coordinate (Tiled coordinate, 0 = left)
    /// * `y` - Grid Y coordinate (Tiled coordinate, 0 = top)
    /// * `tile_size` - Size of tiles in world units
    pub fn grid_to_world(&self, x: u32, y: u32, tile_size: Vec2) -> Vec2 {
        // Flip Y: Tiled y=0 is top row, which maps to highest Y in Bevy
        let flipped_y = self.height - 1 - y;
        Vec2::new(
            (x as f32 + 0.5) * tile_size.x,
            (flipped_y as f32 + 0.5) * tile_size.y,
        )
    }

    /// Iterate all non-empty tiles with their world positions.
    ///
    /// Returns `(world_pos, tile_instance)` tuples where `world_pos` is the center
    /// of the tile in Bevy's coordinate system (Y-up, bottom-left origin).
    ///
    /// This is the recommended iterator for Layer 3 physics plugins.
    pub fn iter_tiles_world(&self, tile_size: Vec2) -> impl Iterator<Item = (Vec2, &TileInstance)> {
        let width = self.width;
        let height = self.height;
        self.tiles
            .iter()
            .enumerate()
            .filter_map(move |(idx, tile)| {
                tile.as_ref().map(|t| {
                    let x = (idx as u32) % width;
                    let y = (idx as u32) / width;
                    // Flip Y: Tiled y=0 is top row, which maps to highest Y in Bevy
                    let flipped_y = height - 1 - y;
                    let world_pos = Vec2::new(
                        (x as f32 + 0.5) * tile_size.x,
                        (flipped_y as f32 + 0.5) * tile_size.y,
                    );
                    (world_pos, t)
                })
            })
    }
}

/// Pre-processed tile data (NOT a component, stored in `TileLayerData`).
///
/// Contains all data needed for rendering and physics, pre-resolved from the map.
#[derive(Debug, Clone, Reflect)]
pub struct TileInstance {
    /// Original GID from map (for reference/debugging)
    pub gid: u32,

    /// Which tileset this tile belongs to
    pub tileset_handle: Handle<TiledTilesetAsset>,

    /// Local tile ID within the tileset (0-based)
    pub tile_id: u32,

    /// Horizontal flip flag
    pub flipped_h: bool,

    /// Vertical flip flag
    pub flipped_v: bool,

    /// Diagonal flip flag (used for rotation in some contexts)
    pub flipped_d: bool,
}
