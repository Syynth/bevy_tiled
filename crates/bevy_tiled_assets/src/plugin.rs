use bevy::prelude::*;

use crate::assets::{
    map::TiledMapAsset, template::TiledTemplateAsset, tileset::TiledTilesetAsset,
    world::TiledWorldAsset,
};
use crate::loaders::{
    TiledResourceCache, map::TiledMapAssetLoader, template::TiledTemplateAssetLoader,
    tileset::TiledTilesetAssetLoader, world::TiledWorldAssetLoader,
};

/// Plugin that registers all Tiled asset types and loaders
///
/// This plugin enables loading Tiled files (.tmx, .tsx, .tx, .world) as Bevy assets.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_tiled_assets::BevyTiledAssetsPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(BevyTiledAssetsPlugin)
///     .run();
/// ```
///
/// # What this plugin does
///
/// - Registers 4 asset types: `TiledMapAsset`, `TiledTilesetAsset`, `TiledTemplateAsset`, `TiledWorldAsset`
/// - Registers 4 asset loaders for `.tmx`, `.tsx`, `.tx`, and `.world` files
/// - Initializes a shared resource cache to prevent duplicate file parsing
///
/// # What this plugin does NOT do
///
/// - Entity spawning (that's Layer 2 - `bevy_tiled_core`)
/// - Rendering (that's Layer 3 - `bevy_tiled_render` or custom user code)
/// - Physics integration (that's Layer 3 - `bevy_tiled_physics`)
///
/// This is a **Layer 1** plugin: pure asset loading with no ECS concerns.
pub struct BevyTiledAssetsPlugin;

impl Plugin for BevyTiledAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Initialize shared resource cache for the tiled::Loader
        // This prevents re-parsing the same .tsx or .tx file multiple times
        let cache = TiledResourceCache::default();

        // Register all 4 asset types
        app.init_asset::<TiledMapAsset>()
            .init_asset::<TiledTilesetAsset>()
            .init_asset::<TiledTemplateAsset>()
            .init_asset::<TiledWorldAsset>();

        // Register all 4 asset loaders with shared cache
        app.register_asset_loader(TiledTilesetAssetLoader {
            cache: cache.clone(),
        })
        .register_asset_loader(TiledTemplateAssetLoader {
            cache: cache.clone(),
        })
        .register_asset_loader(TiledMapAssetLoader {
            cache: cache.clone(),
        })
        .register_asset_loader(TiledWorldAssetLoader {
            cache: cache.clone(),
        });

        // Store cache as resource for potential future use
        app.insert_resource(cache);
    }
}
