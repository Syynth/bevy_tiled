//! Spawn context for accessing asset data during entity spawning.

use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::{TiledMapAsset, TiledTemplateAsset, TiledTilesetAsset};
use std::ops::Range;
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

    /// Cached GID ranges for fast tileset lookup
    /// (GID range, tileset handle)
    tileset_ranges: Vec<(Range<u32>, Handle<TiledTilesetAsset>)>,
}

impl<'a> SpawnContext<'a> {
    /// Create a new spawn context.
    ///
    /// Prepares cached GID ranges for fast tileset lookups.
    pub fn new(
        map_asset: &'a TiledMapAsset,
        tileset_assets: &'a Assets<TiledTilesetAsset>,
        template_assets: &'a Assets<TiledTemplateAsset>,
        registry: &'a crate::properties::TiledClassRegistry,
    ) -> Self {
        // Build tileset ranges for GID lookup
        let mut tileset_ranges = Vec::new();

        // Sort tilesets by first_gid to build ranges
        let mut sorted_tilesets: Vec<_> = map_asset.tilesets.iter().collect();
        sorted_tilesets.sort_by_key(|(first_gid, _)| **first_gid);

        for (i, (first_gid, tileset_ref)) in sorted_tilesets.iter().enumerate() {
            let start_gid = **first_gid;

            // Calculate end GID (start of next tileset, or max if last)
            let end_gid = if i + 1 < sorted_tilesets.len() {
                *sorted_tilesets[i + 1].0
            } else {
                u32::MAX
            };

            tileset_ranges.push((start_gid..end_gid, tileset_ref.handle.clone()));
        }

        Self {
            map_asset,
            tileset_assets,
            template_assets,
            registry,
            tileset_ranges,
        }
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

    /// Resolve a GID to its tileset handle and local tile ID.
    ///
    /// Returns `None` if the GID doesn't match any tileset.
    ///
    /// # Arguments
    ///
    /// * `gid` - The global tile ID from the map
    ///
    /// # Returns
    ///
    /// * `Some((tileset_handle, local_tile_id))` if found
    /// * `None` if GID doesn't match any tileset
    pub fn resolve_gid(&self, gid: u32) -> Option<(Handle<TiledTilesetAsset>, u32)> {
        // GID 0 means empty tile
        if gid == 0 {
            return None;
        }

        // Strip flip flags (top 3 bits)
        let clean_gid = gid & !0xE0000000;

        // Find tileset containing this GID
        for (range, handle) in &self.tileset_ranges {
            if range.contains(&clean_gid) {
                let local_id = clean_gid - range.start;
                return Some((handle.clone(), local_id));
            }
        }

        None
    }

    /// Extract flip flags from a GID.
    ///
    /// Tiled encodes flip flags in the top 3 bits of the GID.
    ///
    /// # Returns
    ///
    /// `(flipped_horizontally, flipped_vertically, flipped_diagonally)`
    pub fn extract_flip_flags(gid: u32) -> (bool, bool, bool) {
        const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
        const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
        const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;

        let flipped_h = (gid & FLIPPED_HORIZONTALLY_FLAG) != 0;
        let flipped_v = (gid & FLIPPED_VERTICALLY_FLAG) != 0;
        let flipped_d = (gid & FLIPPED_DIAGONALLY_FLAG) != 0;

        (flipped_h, flipped_v, flipped_d)
    }
}
