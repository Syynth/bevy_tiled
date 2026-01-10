use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use normalize_path::NormalizePath;
use thiserror::Error;

use crate::assets::tileset::TiledTilesetAsset;
use crate::loaders::TiledResourceCache;

/// Asset loader for Tiled tilesets (.tsx files)
///
/// Supports both texture atlas tilesets (single spritesheet) and image collection
/// tilesets (individual images per tile).
#[derive(Default)]
pub struct TiledTilesetAssetLoader {
    pub cache: TiledResourceCache,
}

#[derive(Debug, Error)]
pub enum TilesetLoaderError {
    #[error("Failed to load tileset: {0}")]
    TiledError(#[from] tiled::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl AssetLoader for TiledTilesetAssetLoader {
    type Asset = TiledTilesetAsset;
    type Settings = ();
    type Error = TilesetLoaderError;

    fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            // Parse TSX using tiled crate
            // The tiled loader reads directly from the filesystem
            let asset_path = load_context.asset_path().path();

            // Construct full filesystem path
            // Bevy loads assets from the "assets" directory by default
            let full_path = std::path::Path::new("assets").join(asset_path);

            // Create loader with default cache and filesystem reader
            // TODO: Implement shared cache once we figure out the correct API
            let mut loader = tiled::Loader::new();

            let tileset = loader.load_tsx_tileset(&full_path)?;

            // 3. Determine if texture atlas or image collection
            let (atlas_image, tile_images) = if let Some(ref image) = tileset.image {
                // TEXTURE ATLAS MODE: Single spritesheet
                let image_path =
                    resolve_relative_path(load_context, &image.source.to_string_lossy())?;
                let handle = load_context.load(image_path);
                (Some(handle), HashMap::default())
            } else {
                // IMAGE COLLECTION MODE: Per-tile images
                let mut tile_images = HashMap::new();
                for (tile_id, tile) in tileset.tiles() {
                    if let Some(ref tile_image) = tile.image {
                        let image_path = resolve_relative_path(
                            load_context,
                            &tile_image.source.to_string_lossy(),
                        )?;
                        let handle = load_context.load(image_path);
                        tile_images.insert(tile_id, handle);
                    }
                }
                (None, tile_images)
            };

            // 4. Extract processed data
            let tile_size = UVec2::new(tileset.tile_width, tileset.tile_height);
            let grid_size = calculate_grid_size(&tileset);
            let spacing = tileset.spacing;
            let margin = tileset.margin;

            // 5. Extract custom properties
            let properties = tileset.properties.clone();

            // Extract per-tile properties
            let tile_properties: HashMap<u32, crate::properties::Properties> = tileset
                .tiles()
                .map(|(tile_id, tile)| (tile_id, tile.properties.clone()))
                .collect();

            // 6. Build asset
            Ok(TiledTilesetAsset {
                tileset,
                atlas_image,
                tile_images,
                tile_size,
                grid_size,
                spacing,
                margin,
                properties,
                tile_properties,
            })
        }
    }

    fn extensions(&self) -> &[&str] {
        &["tsx"]
    }
}

/// Resolve relative path from Tiled file to Bevy asset path
///
/// Tiled uses relative paths like `../path/to/image.png`, but Bevy's asset system
/// expects asset-root-relative paths like `path/to/image.png`.
///
/// This function:
/// 1. Gets the parent directory of the current asset
/// 2. Joins the relative path to the parent
/// 3. Normalizes path separators (Windows `\` → Unix `/`)
///
/// # Arguments
/// * `load_context` - The current asset's load context
/// * `relative_path` - The relative path from the Tiled file (e.g., `../images/tile.png`)
///
/// # Returns
/// * `Ok(String)` - The asset-root-relative path
/// * `Err(TilesetLoaderError)` - If path resolution fails
fn resolve_relative_path(
    load_context: &LoadContext,
    relative_path: &str,
) -> Result<String, TilesetLoaderError> {
    // If the path already starts with "assets/", it's an absolute filesystem path
    // from the tiled loader - just strip the prefix
    if let Some(stripped) = relative_path.strip_prefix("assets/") {
        return Ok(stripped.to_string());
    }

    // Otherwise, resolve relative to the current asset's parent directory
    let parent = load_context.asset_path().path().parent().ok_or_else(|| {
        TilesetLoaderError::InvalidPath(format!(
            "No parent directory for asset: {:?}",
            load_context.asset_path().path()
        ))
    })?;

    let full_path = parent.join(relative_path);

    // Normalize to resolve .. and . components
    // (Path::join does NOT normalize - it just concatenates)
    let normalized = full_path.normalize();

    // Convert to Bevy asset path (forward slashes, no leading slash)
    let asset_path = normalized
        .to_str()
        .ok_or_else(|| {
            TilesetLoaderError::InvalidPath(format!("Invalid UTF-8 in path: {:?}", normalized))
        })?
        .replace('\\', "/");

    Ok(asset_path)
}

/// Calculate grid size (columns, rows) for a tileset
///
/// For texture atlas tilesets, this calculates the grid dimensions from the
/// number of columns and total tile count.
///
/// For image collection tilesets (no columns), this returns `UVec2::ZERO`.
///
/// # Arguments
/// * `tileset` - The parsed tileset from the tiled crate
///
/// # Returns
/// * `UVec2` - Grid size as (columns, rows)
fn calculate_grid_size(tileset: &tiled::Tileset) -> UVec2 {
    if tileset.columns > 0 {
        // Texture atlas: calculate rows from total tiles and columns
        let rows = tileset.tilecount.div_ceil(tileset.columns);
        UVec2::new(tileset.columns, rows)
    } else {
        // Image collection: no grid
        UVec2::ZERO
    }
}
