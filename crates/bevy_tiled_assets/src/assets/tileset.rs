use bevy::{platform::collections::HashMap, prelude::*};

/// Bevy asset wrapper for Tiled tilesets (.tsx files)
///
/// Supports both texture atlas tilesets (single spritesheet) and image collection
/// tilesets (individual images per tile).
#[derive(TypePath, Asset, Debug)]
pub struct TiledTilesetAsset {
    /// Raw Tiled tileset data (PRESERVE AS-IS)
    ///
    /// All original tileset data from the .tsx file is preserved here.
    /// This includes tiles, animations, wang sets, custom properties, etc.
    pub tileset: tiled::Tileset,

    // ===== IMAGE REFERENCES =====
    /// For texture atlas tilesets: single spritesheet image
    ///
    /// This is `Some(handle)` when the tileset uses a single spritesheet image.
    /// For image collection tilesets, this is `None`.
    pub atlas_image: Option<Handle<Image>>,

    /// For image collection tilesets: individual tile images
    ///
    /// Each tile can have its own image file in an image collection tileset.
    /// Key: Local tile ID (0-based, NOT GID)
    /// Value: Handle to the tile's image
    pub tile_images: HashMap<u32, Handle<Image>>,

    // ===== PROCESSED DATA FOR CONVENIENCE =====
    /// Tile size in pixels (width, height)
    ///
    /// Copied from tileset for convenient access without navigating the raw data.
    pub tile_size: UVec2,

    /// Tileset grid dimensions in tiles (columns, rows)
    ///
    /// For texture atlas tilesets, this is calculated from columns and tile count.
    /// For image collection tilesets, this is `UVec2::ZERO`.
    pub grid_size: UVec2,

    /// Spacing between tiles in the atlas (pixels)
    ///
    /// Only relevant for texture atlas tilesets.
    pub spacing: u32,

    /// Margin around the tileset in the atlas (pixels)
    ///
    /// Only relevant for texture atlas tilesets.
    pub margin: u32,
    // ===== CUSTOM PROPERTIES =====
    /// Custom properties set on the tileset in Tiled
    pub properties: crate::properties::Properties,

    /// Custom properties set on individual tiles
    /// Key: Local tile ID (0-based, NOT GID)
    pub tile_properties: HashMap<u32, crate::properties::Properties>,
}

impl TiledTilesetAsset {
    /// Check if this is an image collection tileset (vs. texture atlas)
    ///
    /// Returns `true` if each tile has its own image file, `false` if the tileset
    /// uses a single spritesheet.
    #[inline]
    pub fn is_image_collection(&self) -> bool {
        self.atlas_image.is_none()
    }

    /// Get the image handle for a specific tile
    ///
    /// For texture atlas tilesets, this returns the atlas image (same for all tiles).
    /// For image collection tilesets, this returns the specific tile's image.
    ///
    /// # Arguments
    /// * `local_tile_id` - The local tile ID (0-based, NOT a GID)
    ///
    /// # Returns
    /// * `Some(&Handle<Image>)` - The image handle for this tile
    /// * `None` - If the tile doesn't exist or has no image
    pub fn get_tile_image(&self, local_tile_id: u32) -> Option<&Handle<Image>> {
        if let Some(ref atlas) = self.atlas_image {
            // Texture atlas mode: all tiles share the same image
            Some(atlas)
        } else {
            // Image collection mode: look up the specific tile's image
            self.tile_images.get(&local_tile_id)
        }
    }
}
