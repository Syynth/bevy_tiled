use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use thiserror::Error;

use crate::assets::{
    map::{TiledMapAsset, TilesetReference},
    tileset::TiledTilesetAsset,
};
use crate::loaders::TiledResourceCache;

/// Asset loader for Tiled maps (.tmx files)
///
/// This loader handles all map dependencies:
/// - Tilesets (.tsx files)
/// - Templates (.tx files) referenced by objects
/// - Images for image layers
///
/// It also calculates processed data for infinite maps.
#[derive(Default)]
pub struct TiledMapAssetLoader {
    pub cache: TiledResourceCache,
}

#[derive(Debug, Error)]
pub enum MapLoaderError {
    #[error("Failed to load map: {0}")]
    TiledError(#[from] tiled::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl AssetLoader for TiledMapAssetLoader {
    type Asset = TiledMapAsset;
    type Settings = ();
    type Error = MapLoaderError;

    fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            // Parse TMX using tiled crate
            let asset_path = load_context.asset_path().path();

            // Construct full filesystem path
            // Bevy loads assets from the "assets" directory by default
            let full_path = std::path::Path::new("assets").join(asset_path);

            // Create loader with default cache
            // TODO: Implement shared cache once we figure out the correct API
            let mut loader = tiled::Loader::new();

            let map = loader.load_tmx_map(&full_path)?;

            // 3. Load tileset dependencies
            // Key by tileset_index (iteration order matches tiled's tileset_index())
            let mut tilesets = HashMap::default();
            let mut current_gid = 1u32; // GIDs start at 1

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                // External tileset: load as dependency
                let tileset_path =
                    resolve_relative_path(load_context, &tileset.source.to_string_lossy())?;
                let handle: Handle<TiledTilesetAsset> = load_context.load(tileset_path);

                // Key by tileset_index for direct lookup from LayerTile::tileset_index()
                tilesets.insert(
                    tileset_index as u32,
                    TilesetReference {
                        handle,
                        first_gid: current_gid,
                    },
                );

                // Next tileset's first_gid = current + tile count
                current_gid += tileset.tilecount;
            }

            // 4. Templates are handled internally by tiled crate's ResourceCache
            // when objects are parsed. No need to track them separately.
            let templates = HashMap::default();

            // 5. Load image layer dependencies (recursively searches group layers)
            let mut images = HashMap::default();
            collect_image_layers(&map, load_context, &mut images)?;

            // 6. Calculate processed data
            let (tilemap_size, largest_tile_size, rect) = calculate_map_bounds(&map, &tilesets);

            // 7. Calculate infinite map offsets
            let (tiled_offset, topleft_chunk, bottomright_chunk) =
                calculate_infinite_map_data(&map);

            // 8. Extract and normalize custom properties
            // Normalize FileValue paths to be asset-root-relative (resolves ../foo paths)
            let mut properties = map.properties.clone();
            normalize_property_paths(&mut properties, load_context);

            // 9. Extract and normalize layer properties (recursively searches group layers)
            let mut layer_properties = HashMap::default();
            collect_layer_properties(&map, load_context, &mut layer_properties);

            // 10. Extract and normalize object properties from all object layers (recursively)
            let mut object_properties = HashMap::default();
            collect_object_properties(&map, load_context, &mut object_properties);

            // 11. Build asset
            Ok(TiledMapAsset {
                map,
                tilesets,
                templates,
                images,
                tilemap_size,
                largest_tile_size,
                rect,
                tiled_offset,
                topleft_chunk,
                bottomright_chunk,
                properties,
                layer_properties,
                object_properties,
            })
        }
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}

/// Calculate map bounds and tilemap size
///
/// For finite maps, uses the map dimensions directly.
/// For infinite maps, calculates bounds from chunk data.
///
/// # Arguments
/// * `map` - The parsed Tiled map
/// * `tilesets` - The loaded tileset references
///
/// # Returns
/// * `(tilemap_size, largest_tile_size, rect)` tuple
fn calculate_map_bounds(
    map: &tiled::Map,
    _tilesets: &HashMap<u32, TilesetReference>,
) -> (UVec2, UVec2, Rect) {
    // Find largest tile size across all tilesets
    let largest_tile_size = UVec2::new(map.tile_width, map.tile_height);

    // Note: We can't access tileset asset data here (not loaded yet),
    // so we use the map's default tile size as a safe approximation.
    // Layer 2 can refine this if needed using actual tileset data.

    if map.infinite() {
        // Infinite map: calculate bounds from chunks
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for layer in map.layers() {
            if let Some(tile_layer) = layer.as_tile_layer() {
                match tile_layer {
                    tiled::TileLayer::Finite(_) => {
                        // Shouldn't happen in infinite maps, but handle gracefully
                    }
                    tiled::TileLayer::Infinite(infinite_layer) => {
                        for ((chunk_x, chunk_y), _chunk) in infinite_layer.chunks() {
                            min_x = min_x.min(chunk_x);
                            min_y = min_y.min(chunk_y);
                            max_x = max_x.max(chunk_x);
                            max_y = max_y.max(chunk_y);
                        }
                    }
                }
            }
        }

        // Chunk dimensions are constants in the tiled crate
        let chunk_width = tiled::ChunkData::WIDTH;
        let chunk_height = tiled::ChunkData::HEIGHT;

        // Calculate size in chunks, then convert to tiles
        let chunks_wide = if max_x >= min_x { max_x - min_x + 1 } else { 0 };
        let chunks_tall = if max_y >= min_y { max_y - min_y + 1 } else { 0 };

        let tilemap_size = UVec2::new(
            (chunks_wide * chunk_width as i32) as u32,
            (chunks_tall * chunk_height as i32) as u32,
        );

        let rect_width = tilemap_size.x as f32 * map.tile_width as f32;
        let rect_height = tilemap_size.y as f32 * map.tile_height as f32;
        let rect = Rect::new(0.0, 0.0, rect_width, rect_height);

        (tilemap_size, largest_tile_size, rect)
    } else {
        // Finite map: use map dimensions directly
        let tilemap_size = UVec2::new(map.width, map.height);

        let rect_width = map.width as f32 * map.tile_width as f32;
        let rect_height = map.height as f32 * map.tile_height as f32;
        let rect = Rect::new(0.0, 0.0, rect_width, rect_height);

        (tilemap_size, largest_tile_size, rect)
    }
}

/// Calculate infinite map offset and chunk bounds
///
/// For infinite maps, finds the topmost-left and bottommost-right chunks,
/// and calculates an offset to shift the entire map into positive coordinate space.
///
/// For finite maps, returns zero offset and (0,0) chunk bounds.
///
/// # Arguments
/// * `map` - The parsed Tiled map
///
/// # Returns
/// * `(tiled_offset, topleft_chunk, bottomright_chunk)` tuple
fn calculate_infinite_map_data(map: &tiled::Map) -> (Vec2, (i32, i32), (i32, i32)) {
    if map.infinite() {
        let mut min_chunk_x = i32::MAX;
        let mut min_chunk_y = i32::MAX;
        let mut max_chunk_x = i32::MIN;
        let mut max_chunk_y = i32::MIN;

        // Chunk dimensions are constants
        let chunk_width = tiled::ChunkData::WIDTH;
        let chunk_height = tiled::ChunkData::HEIGHT;

        for layer in map.layers() {
            if let Some(tile_layer) = layer.as_tile_layer()
                && let tiled::TileLayer::Infinite(infinite_layer) = tile_layer
            {
                for ((chunk_x, chunk_y), _chunk) in infinite_layer.chunks() {
                    min_chunk_x = min_chunk_x.min(chunk_x);
                    min_chunk_y = min_chunk_y.min(chunk_y);
                    max_chunk_x = max_chunk_x.max(chunk_x);
                    max_chunk_y = max_chunk_y.max(chunk_y);
                }
            }
        }

        // Calculate offset to shift map into positive space
        // If min chunk is negative, we need to offset by that amount
        let offset_x = if min_chunk_x < 0 {
            -min_chunk_x as f32 * chunk_width as f32 * map.tile_width as f32
        } else {
            0.0
        };

        let offset_y = if min_chunk_y < 0 {
            -min_chunk_y as f32 * chunk_height as f32 * map.tile_height as f32
        } else {
            0.0
        };

        let tiled_offset = Vec2::new(offset_x, offset_y);
        let topleft_chunk = (min_chunk_x, min_chunk_y);
        let bottomright_chunk = (max_chunk_x, max_chunk_y);

        (tiled_offset, topleft_chunk, bottomright_chunk)
    } else {
        // Finite map: no offset needed
        (Vec2::ZERO, (0, 0), (0, 0))
    }
}

/// Recursively collect image layers and load their images.
///
/// Tiled maps can have image layers nested inside group layers. This function
/// recursively traverses all layers to find and load all image dependencies.
fn collect_image_layers(
    map: &tiled::Map,
    load_context: &mut LoadContext,
    images: &mut HashMap<u32, Handle<Image>>,
) -> Result<(), MapLoaderError> {
    fn collect_from_layers<'a>(
        layers: impl Iterator<Item = tiled::Layer<'a>>,
        load_context: &mut LoadContext,
        images: &mut HashMap<u32, Handle<Image>>,
    ) -> Result<(), MapLoaderError> {
        for layer in layers {
            if let Some(image_layer) = layer.as_image_layer() {
                if let Some(ref image) = image_layer.image {
                    let image_path =
                        resolve_relative_path(load_context, &image.source.to_string_lossy())?;
                    let handle: Handle<Image> = load_context.load(image_path);
                    images.insert(layer.id(), handle);
                }
            } else if let Some(group) = layer.as_group_layer() {
                // Recursively process group layer children
                collect_from_layers(group.layers(), load_context, images)?;
            }
        }
        Ok(())
    }

    collect_from_layers(map.layers(), load_context, images)
}

/// Recursively collect layer properties from all layers including nested groups.
fn collect_layer_properties(
    map: &tiled::Map,
    load_context: &LoadContext,
    layer_properties: &mut HashMap<u32, tiled::Properties>,
) {
    fn collect_from_layers<'a>(
        layers: impl Iterator<Item = tiled::Layer<'a>>,
        load_context: &LoadContext,
        layer_properties: &mut HashMap<u32, tiled::Properties>,
    ) {
        for layer in layers {
            if !layer.properties.is_empty() {
                let mut props = layer.properties.clone();
                normalize_property_paths(&mut props, load_context);
                layer_properties.insert(layer.id(), props);
            }

            // Recursively process group layer children
            if let Some(group) = layer.as_group_layer() {
                collect_from_layers(group.layers(), load_context, layer_properties);
            }
        }
    }

    collect_from_layers(map.layers(), load_context, layer_properties);
}

/// Recursively collect object properties from all object layers including nested groups.
fn collect_object_properties(
    map: &tiled::Map,
    load_context: &LoadContext,
    object_properties: &mut HashMap<u32, tiled::Properties>,
) {
    fn collect_from_layers<'a>(
        layers: impl Iterator<Item = tiled::Layer<'a>>,
        load_context: &LoadContext,
        object_properties: &mut HashMap<u32, tiled::Properties>,
    ) {
        for layer in layers {
            if let Some(object_layer) = layer.as_object_layer() {
                for object in object_layer.objects() {
                    if !object.properties.is_empty() {
                        let mut props = object.properties.clone();
                        normalize_property_paths(&mut props, load_context);
                        object_properties.insert(object.id(), props);
                    }
                }
            } else if let Some(group) = layer.as_group_layer() {
                // Recursively process group layer children
                collect_from_layers(group.layers(), load_context, object_properties);
            }
        }
    }

    collect_from_layers(map.layers(), load_context, object_properties);
}

/// Resolve relative path from Tiled file to Bevy asset path
///
/// Tiled uses relative paths like `../path/to/tileset.tsx`, but Bevy's asset system
/// expects asset-root-relative paths like `path/to/tileset.tsx`.
///
/// This function:
/// 1. Gets the parent directory of the current asset
/// 2. Joins the relative path to the parent
/// 3. Normalizes path separators (Windows `\` → Unix `/`)
///
/// # Arguments
/// * `load_context` - The current asset's load context
/// * `relative_path` - The relative path from the Tiled file (e.g., `../tilesets/dungeon.tsx`)
///
/// # Returns
/// * `Ok(String)` - The asset-root-relative path
/// * `Err(MapLoaderError)` - If path resolution fails
fn resolve_relative_path(
    load_context: &LoadContext,
    relative_path: &str,
) -> Result<String, MapLoaderError> {
    // If the path already starts with "assets/", it's an absolute filesystem path
    // from the tiled loader - just strip the prefix
    if let Some(stripped) = relative_path.strip_prefix("assets/") {
        return Ok(stripped.to_string());
    }

    // Otherwise, resolve relative to the current asset's parent directory
    let parent = load_context.asset_path().path().parent().ok_or_else(|| {
        MapLoaderError::InvalidPath(format!(
            "No parent directory for asset: {:?}",
            load_context.asset_path().path()
        ))
    })?;

    let full_path = parent.join(relative_path);

    // Convert to Bevy asset path (forward slashes, no leading slash)
    let asset_path = full_path
        .to_str()
        .ok_or_else(|| {
            MapLoaderError::InvalidPath(format!("Invalid UTF-8 in path: {:?}", full_path))
        })?
        .replace('\\', "/");

    Ok(asset_path)
}

/// Normalize all `FileValue` paths in properties to be asset-root-relative.
///
/// Tiled stores file references as relative paths (e.g., `../transitions/fade.toml`).
/// Bevy's `AssetServer` rejects paths with `..` components for security reasons.
/// This function resolves relative paths to absolute asset-root-relative paths.
///
/// Handles nested `ClassValue` properties recursively.
///
/// # Arguments
/// * `properties` - The properties map to normalize (modified in place)
/// * `load_context` - The current asset's load context for path resolution
fn normalize_property_paths(properties: &mut tiled::Properties, load_context: &LoadContext) {
    for (_key, value) in properties.iter_mut() {
        normalize_property_value(value, load_context);
    }
}

/// Normalize a single `PropertyValue`, handling `FileValue` and nested `ClassValue`.
fn normalize_property_value(value: &mut tiled::PropertyValue, load_context: &LoadContext) {
    match value {
        tiled::PropertyValue::FileValue(path) => {
            // Resolve relative path to asset-root-relative
            if let Ok(resolved) = resolve_relative_path(load_context, path) {
                *path = resolved;
            }
            // If resolution fails, keep original path (will error at load time with better context)
        }
        tiled::PropertyValue::ClassValue { properties, .. } => {
            // Recursively normalize nested class properties
            normalize_property_paths(properties, load_context);
        }
        // Other property types don't need normalization
        _ => {}
    }
}
