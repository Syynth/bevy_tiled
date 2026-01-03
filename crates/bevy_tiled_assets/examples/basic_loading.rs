//! Basic example demonstrating loading Tiled assets
//!
//! This example shows how to load and access:
//! - Tilesets (.tsx)
//! - Maps (.tmx)
//! - Worlds (.world)
//!
//! Run with: `cargo run --example basic_loading`

use bevy::{log::LogPlugin, prelude::*};
use bevy_tiled_assets::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            LogPlugin::default(),
            ImagePlugin::default(),
        ))
        .add_plugins(BevyTiledAssetsPlugin)
        .add_systems(Startup, load_assets)
        .add_systems(Update, check_assets_loaded)
        .run();
}

#[derive(Resource)]
struct LoadedAssets {
    tileset: Handle<TiledTilesetAsset>,
    map: Handle<TiledMapAsset>,
    world: Handle<TiledWorldAsset>,
    logged: bool,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading Tiled assets...");

    // Load each asset type
    let tileset = asset_server.load("orthogonal_1.tsx");
    let map = asset_server.load("simple_map.tmx");
    let world = asset_server.load("world.world");

    commands.insert_resource(LoadedAssets {
        tileset,
        map,
        world,
        logged: false,
    });

    info!("Asset loading initiated");
}

fn check_assets_loaded(
    mut loaded_assets: ResMut<LoadedAssets>,
    tilesets: Res<Assets<TiledTilesetAsset>>,
    maps: Res<Assets<TiledMapAsset>>,
    worlds: Res<Assets<TiledWorldAsset>>,
    mut exit: MessageWriter<AppExit>,
) {
    // Only log once when all assets are ready
    if loaded_assets.logged {
        return;
    }

    let tileset = tilesets.get(&loaded_assets.tileset);
    let map = maps.get(&loaded_assets.map);
    let world = worlds.get(&loaded_assets.world);

    // Check if all assets are loaded
    if tileset.is_none() || map.is_none() || world.is_none() {
        return;
    }

    info!("\n=== All Assets Loaded Successfully ===\n");

    // tileset properties
    if let Some(tileset_asset) = tileset {
        info!("📦 TILESET:");
        info!("  Name: {}", tileset_asset.tileset.name);
        info!(
            "  Tile Size: {}x{}",
            tileset_asset.tile_size.x, tileset_asset.tile_size.y
        );
        info!(
            "  Grid Size: {}x{} tiles",
            tileset_asset.grid_size.x, tileset_asset.grid_size.y
        );
        info!("  Tile Count: {}", tileset_asset.tileset.tilecount);
        info!("  Columns: {}", tileset_asset.tileset.columns);
        info!("  Spacing: {}", tileset_asset.spacing);
        info!("  Margin: {}", tileset_asset.margin);
        info!(
            "  Image Collection: {}",
            tileset_asset.is_image_collection()
        );

        if let Some(_atlas) = &tileset_asset.atlas_image {
            info!("  Atlas Image: Loaded");
        }

        info!(
            "  Individual Tile Images: {}\n",
            tileset_asset.tile_images.len()
        );

        // custom properties
        if !tileset_asset.properties.is_empty() {
            info!("  Tileset Properties:");
            for (key, value) in &tileset_asset.properties {
                info!("    {}: {:?}", key, value);
            }
        }

        // tile properties
        let tiles_with_props = tileset_asset.tile_properties.len();
        if tiles_with_props > 0 {
            info!("  Tiles with custom properties: {}", tiles_with_props);
        }
    }

    // map properties
    if let Some(map_asset) = map {
        info!("🗺️  MAP:");
        info!(
            "  Dimensions: {}x{} tiles",
            map_asset.map.width, map_asset.map.height
        );
        info!(
            "  Tile Size: {}x{} pixels",
            map_asset.map.tile_width, map_asset.map.tile_height
        );
        info!("  Orientation: {:?}", map_asset.map.orientation);
        info!("  Infinite: {}", map_asset.map.infinite());
        info!("  Tilemap Size: {:?}", map_asset.tilemap_size);
        info!("  Largest Tile Size: {:?}", map_asset.largest_tile_size);
        info!("  Bounding Rect: {:?}", map_asset.rect);
        info!("  Background Color: {:?}", map_asset.map.background_color);

        info!("  Tilesets: {}", map_asset.tilesets.len());
        for (gid, tileset_ref) in &map_asset.tilesets {
            info!("    - First GID: {}", gid);
            info!("      (Reference to tileset asset) {:?}", tileset_ref);
        }

        info!("  Layers: {}", map_asset.map.layers().len());
        for layer in map_asset.map.layers() {
            info!("    - Layer '{}' (id: {})", layer.name, layer.id());
            match layer.layer_type() {
                tiled::LayerType::Tiles(_) => info!("      Type: Tile Layer"),
                tiled::LayerType::Objects(_) => info!("      Type: Object Layer"),
                tiled::LayerType::Image(_) => info!("      Type: Image Layer"),
                tiled::LayerType::Group(_) => info!("      Type: Group Layer"),
            }
        }

        info!("  Templates: {}", map_asset.templates.len());
        info!("  Images: {}\n", map_asset.images.len());

        // Log map properties
        if !map_asset.properties.is_empty() {
            info!("  Map Properties:");
            for (key, value) in &map_asset.properties {
                info!("    {}: {:?}", key, value);
            }
        }
    }

    // world properties
    if let Some(world_asset) = world {
        info!("🌍 WORLD:");
        info!("  Maps in world: {}", world_asset.map_count());
        for (map_name, _handle) in &world_asset.maps {
            info!("    - {}", map_name);
        }
    }

    info!("\n=== Example Complete ===");
    info!("All assets loaded and logged successfully!\n");

    loaded_assets.logged = true;

    exit.write(AppExit::Success);
}
