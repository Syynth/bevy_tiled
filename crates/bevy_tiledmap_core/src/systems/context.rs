//! Spawn context for accessing asset data during entity spawning.

use bevy::{asset::AssetServer, prelude::*};
use bevy_tiledmap_assets::assets::map::TilesetReference;
use bevy_tiledmap_assets::prelude::{TiledMapAsset, TiledTemplateAsset, TiledTilesetAsset};
use tiled::Properties;

/// Read-only context providing access to asset data during spawning.
///
/// Used internally by the spawning system. Not passed to Layer 3 events.
pub struct SpawnContext<'a> {
    /// The map asset being spawned
    pub map_asset: &'a TiledMapAsset,

    /// Access to all tileset assets
    pub tileset_assets: &'a Assets<TiledTilesetAsset>,

    /// Access to all template assets (for property merging)
    pub template_assets: &'a Assets<TiledTemplateAsset>,

    /// `TiledClass` registry for component deserialization
    pub registry: &'a crate::properties::TiledClassRegistry,

    /// Asset server for loading `Handle<T>` fields during deserialization
    pub asset_server: &'a AssetServer,
}

impl<'a> SpawnContext<'a> {
    /// Create a new spawn context.
    pub fn new(
        map_asset: &'a TiledMapAsset,
        tileset_assets: &'a Assets<TiledTilesetAsset>,
        template_assets: &'a Assets<TiledTemplateAsset>,
        registry: &'a crate::properties::TiledClassRegistry,
        asset_server: &'a AssetServer,
    ) -> Self {
        Self {
            map_asset,
            tileset_assets,
            template_assets,
            registry,
            asset_server,
        }
    }

    /// Get tileset reference by index.
    ///
    /// The index corresponds to `LayerTile::tileset_index()` from the tiled crate.
    ///
    /// # Arguments
    ///
    /// * `tileset_index` - The tileset index from the tile
    ///
    /// # Returns
    ///
    /// * `Some(&TilesetReference)` if found
    /// * `None` if index doesn't exist
    pub fn get_tileset_by_index(&self, tileset_index: u32) -> Option<&TilesetReference> {
        self.map_asset.tilesets.get(&tileset_index)
    }

    /// Get merged properties for an object (template + object override).
    ///
    /// Returns the normalized properties from the map asset, where file paths have been
    /// resolved to asset-root-relative paths.
    ///
    /// In tiled v0.15, template properties are automatically merged during map parsing.
    /// The returned properties contain any template inheritance, with file paths normalized.
    ///
    /// # Arguments
    ///
    /// * `object_id` - The object ID from the map
    ///
    /// # Returns
    ///
    /// The object's normalized properties if found
    pub fn get_object_properties(&self, object_id: u32) -> Option<&Properties> {
        self.map_asset.object_properties.get(&object_id)
    }
}
