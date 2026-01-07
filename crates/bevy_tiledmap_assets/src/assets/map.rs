use bevy::{platform::collections::HashMap, prelude::*};

use crate::assets::{template::TiledTemplateAsset, tileset::TiledTilesetAsset};

#[derive(TypePath, Asset, Debug)]
pub struct TiledMapAsset {
    /// The raw Tiled map data (PRESERVE AS-IS)
    pub map: tiled::Map,

    // ===== BEVY ASSET REFERENCES =====
    /// Tileset handles (Bevy asset system)
    /// Key: Tileset index (matches `LayerTile::tileset_index()`)
    pub tilesets: HashMap<u32, TilesetReference>,

    /// Template handles (Bevy asset system)
    /// Key: Template source path
    pub templates: HashMap<String, Handle<TiledTemplateAsset>>,

    /// Image layer images (Bevy asset system)
    /// Key: Layer ID
    pub images: HashMap<u32, Handle<Image>>,

    // ===== PROCESSED DATA FOR BEVY =====
    /// Map size in tiles (for tilemap systems)
    pub tilemap_size: UVec2,

    /// Largest tile size across all tilesets
    pub largest_tile_size: UVec2,

    /// Map bounding box in pixels
    pub rect: Rect,

    // ===== INFINITE MAP SUPPORT =====
    pub tiled_offset: Vec2,
    pub topleft_chunk: (i32, i32),
    pub bottomright_chunk: (i32, i32),
    // ===== CUSTOM PROPERTIES =====
    /// Custom properties set on the map in Tiled
    pub properties: crate::properties::Properties,

    /// Custom properties set on layers
    /// Key: Layer ID
    /// Value: Properties for that layer
    pub layer_properties: HashMap<u32, crate::properties::Properties>,

    /// Custom properties set on objects
    /// Key: Object ID
    /// Value: Properties for that object
    pub object_properties: HashMap<u32, crate::properties::Properties>,
}

#[derive(Debug, Clone)]
pub struct TilesetReference {
    /// Bevy asset handle to the tileset
    pub handle: Handle<TiledTilesetAsset>,
    /// First GID of this tileset in the map
    pub first_gid: u32,
}
