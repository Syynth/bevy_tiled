use bevy::{platform::collections::HashMap, prelude::*};

use crate::assets::map::TiledMapAsset;

/// Bevy asset wrapper for Tiled worlds (.world files)
///
/// Worlds contain multiple maps and their positioning in a larger game world.
#[derive(TypePath, Asset, Debug)]
pub struct TiledWorldAsset {
    /// Raw Tiled world data (PRESERVE AS-IS)
    ///
    /// All original world data from the .world file is preserved here.
    /// This includes world properties and map positions.
    pub world: tiled::World,

    // ===== ASSET REFERENCES =====
    /// Map asset handles
    ///
    /// All maps referenced by this world are loaded as dependencies.
    /// Key: Map file name (as specified in the world file)
    /// Value: Handle to the loaded map asset
    pub maps: HashMap<String, Handle<TiledMapAsset>>,
}

impl TiledWorldAsset {
    /// Get the number of maps in this world
    #[inline]
    pub fn map_count(&self) -> usize {
        self.maps.len()
    }

    /// Check if a specific map is in this world
    ///
    /// # Arguments
    /// * `map_name` - The map file name to check for
    ///
    /// # Returns
    /// * `true` if the map is in this world, `false` otherwise
    #[inline]
    pub fn contains_map(&self, map_name: &str) -> bool {
        self.maps.contains_key(map_name)
    }

    /// Get the handle for a specific map
    ///
    /// # Arguments
    /// * `map_name` - The map file name
    ///
    /// # Returns
    /// * `Some(&Handle<TiledMapAsset>)` - The map handle
    /// * `None` - If the map doesn't exist in this world
    #[inline]
    pub fn get_map(&self, map_name: &str) -> Option<&Handle<TiledMapAsset>> {
        self.maps.get(map_name)
    }
}
