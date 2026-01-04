use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use thiserror::Error;

use crate::assets::{map::TiledMapAsset, world::TiledWorldAsset};
use crate::loaders::TiledResourceCache;

/// Asset loader for Tiled worlds (.world files)
///
/// Worlds contain multiple maps and automatically load all referenced maps as dependencies.
#[derive(Default)]
pub struct TiledWorldAssetLoader {
    pub cache: TiledResourceCache,
}

#[derive(Debug, Error)]
pub enum WorldLoaderError {
    #[error("Failed to load world: {0}")]
    TiledError(#[from] tiled::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl AssetLoader for TiledWorldAssetLoader {
    type Asset = TiledWorldAsset;
    type Settings = ();
    type Error = WorldLoaderError;

    fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            // Parse .world file using tiled crate
            let asset_path = load_context.asset_path().path();

            // Construct full filesystem path
            // Bevy loads assets from the "assets" directory by default
            let full_path = std::path::Path::new("assets").join(asset_path);

            // Create loader with default cache
            // TODO: Implement shared cache once we figure out the correct API
            let mut loader = tiled::Loader::new();

            let world = loader.load_world(&full_path)?;

            // 3. Load all map dependencies
            let mut maps = HashMap::default();
            for map_ref in &world.maps {
                // Resolve relative path to the map file
                let map_path = resolve_relative_path(load_context, &map_ref.filename)?;
                let handle: Handle<TiledMapAsset> = load_context.load(map_path);

                // Use the map file name as the key
                maps.insert(map_ref.filename.clone(), handle);
            }

            // 4. Build asset
            Ok(TiledWorldAsset { world, maps })
        }
    }

    fn extensions(&self) -> &[&str] {
        &["world"]
    }
}

/// Resolve relative path from Tiled file to Bevy asset path
///
/// Tiled uses relative paths like `../path/to/map.tmx`, but Bevy's asset system
/// expects asset-root-relative paths like `path/to/map.tmx`.
///
/// This function:
/// 1. Gets the parent directory of the current asset
/// 2. Joins the relative path to the parent
/// 3. Normalizes path separators (Windows `\` → Unix `/`)
///
/// # Arguments
/// * `load_context` - The current asset's load context
/// * `relative_path` - The relative path from the Tiled file (e.g., `../maps/level1.tmx`)
///
/// # Returns
/// * `Ok(String)` - The asset-root-relative path
/// * `Err(WorldLoaderError)` - If path resolution fails
fn resolve_relative_path(
    load_context: &LoadContext,
    relative_path: &str,
) -> Result<String, WorldLoaderError> {
    // If the path already starts with "assets/", it's an absolute filesystem path
    // from the tiled loader - just strip the prefix
    if let Some(stripped) = relative_path.strip_prefix("assets/") {
        return Ok(stripped.to_string());
    }

    // Otherwise, resolve relative to the current asset's parent directory
    let parent = load_context.asset_path().path().parent().ok_or_else(|| {
        WorldLoaderError::InvalidPath(format!(
            "No parent directory for asset: {:?}",
            load_context.asset_path().path()
        ))
    })?;

    let full_path = parent.join(relative_path);

    // Convert to Bevy asset path (forward slashes, no leading slash)
    let asset_path = full_path
        .to_str()
        .ok_or_else(|| {
            WorldLoaderError::InvalidPath(format!("Invalid UTF-8 in path: {:?}", full_path))
        })?
        .replace('\\', "/");

    Ok(asset_path)
}
