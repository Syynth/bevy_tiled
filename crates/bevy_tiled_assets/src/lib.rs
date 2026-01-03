pub mod assets;
pub mod loaders;
pub mod plugin;

pub mod properties;

// Re-export the plugin for convenience
pub use plugin::BevyTiledAssetsPlugin;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types for easy access.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_tiled_assets::prelude::*;
///
/// fn my_system(maps: Res<Assets<TiledMapAsset>>) {
///     // Use Tiled assets...
/// }
/// ```
pub mod prelude {
    pub use crate::assets::{
        map::{TiledMapAsset, TilesetReference},
        template::TiledTemplateAsset,
        tileset::TiledTilesetAsset,
        world::TiledWorldAsset,
    };
    pub use crate::plugin::BevyTiledAssetsPlugin;
}
