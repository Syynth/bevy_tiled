//! Spawn context for accessing asset data during entity spawning.

use bevy::prelude::*;
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
}

impl<'a> SpawnContext<'a> {
    /// Create a new spawn context.
    pub fn new(
        map_asset: &'a TiledMapAsset,
        tileset_assets: &'a Assets<TiledTilesetAsset>,
        template_assets: &'a Assets<TiledTemplateAsset>,
        registry: &'a crate::properties::TiledClassRegistry,
    ) -> Self {
        Self {
            map_asset,
            tileset_assets,
            template_assets,
            registry,
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
    /// In tiled v0.15, template properties are automatically merged during map parsing,
    /// so this method simply returns the object's properties which already contain
    /// any template inheritance.
    ///
    /// # Arguments
    ///
    /// * `object` - The object from the map
    ///
    /// # Returns
    ///
    /// A reference to the object's properties (already merged with template if applicable)
    pub fn get_merged_object_properties<'b>(
        &self,
        object: &'b tiled::ObjectData,
    ) -> &'b Properties {
        // NOTE: The tiled crate (v0.15) automatically merges template properties
        // during map parsing, so object.properties already contains the merged result.
        // Template properties serve as defaults, and object properties override them.
        &object.properties
    }
}
