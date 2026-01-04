//! Object components.

use bevy::prelude::*;
use bevy_tiled_assets::prelude::TiledTilesetAsset;

/// Tiled's original object ID.
///
/// Useful for looking up object-specific data (like properties) from the `TiledMapAsset`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct ObjectId(pub u32);

/// Object component with pre-computed shape data.
///
/// Vertices are pre-computed during spawning (NOT raw points from Tiled).
/// Layer 3 physics/rendering plugins can use this data directly without recomputation.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub enum TiledObject {
    /// Point object (no dimensions)
    Point,

    /// Rectangle object
    Rectangle { width: f32, height: f32 },

    /// Ellipse object
    Ellipse { width: f32, height: f32 },

    /// Polygon object with pre-computed vertices
    Polygon {
        /// Pre-computed vertices (NOT raw f64 points from Tiled)
        vertices: Vec<Vec2>,
    },

    /// Polyline object with pre-computed vertices
    Polyline {
        /// Pre-computed vertices (NOT raw f64 points from Tiled)
        vertices: Vec<Vec2>,
    },

    /// Tile object (references a tile from a tileset)
    Tile {
        /// Local tile ID in tileset
        tile_id: u32,

        /// Which tileset (for accessing tile properties, collision shapes, etc.)
        tileset_handle: Handle<TiledTilesetAsset>,

        /// Object width (may differ from tile width)
        width: f32,

        /// Object height (may differ from tile height)
        height: f32,
    },

    /// Text object (placeholder for now)
    Text {
        // Phase 3+: text content, font, alignment, etc.
    },
}
