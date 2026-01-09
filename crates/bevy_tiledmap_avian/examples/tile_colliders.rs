//! Example demonstrating automatic tile collider generation from tileset collision shapes.
//!
//! This example shows how `bevy_tiledmap_avian` can automatically generate optimized physics
//! colliders from tiles that have collision shapes defined in their tileset.
//!
//! # How to Create a Tileset with Collision Shapes
//!
//! 1. Open Tiled and create/open a tileset
//! 2. Select a tile in the Tileset Editor
//! 3. Click "View" → "Tile Collision Editor" (or press Ctrl+Shift+O)
//! 4. Draw collision shapes on the tile using the Rectangle, Polygon, or Polyline tools
//! 5. Save the tileset
//!
//! # Rectangle Merging Optimization
//!
//! The `CompoundMerged` strategy (default) automatically merges contiguous rectangular
//! tiles into larger shapes, drastically reducing collider count:
//! - 10x10 grid of collision tiles → ~10-20 merged rectangles (instead of 100)
//! - Large platforms → Single long rectangle (instead of many small ones)
//!
//! # Alternative Strategies
//!
//! - `PerTileEntity`: Individual entities per tile (for moving/destructible terrain)
//! - `CompoundChunked`: Chunked compounds for large/infinite maps
//! - `Disabled`: No tile colliders (only object colliders)

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::*;
use bevy_tiledmap_avian::prelude::*;
use bevy_tiledmap_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin) // Show collision shapes
        .add_plugins(TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::default())
        .add_plugins(TiledmapAvianPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Load a map with tiles that have collision shapes defined
    // NOTE: You need to create this map in Tiled with a tileset that has collision shapes
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        TiledMap {
            handle: asset_server.load("maps/tile_colliders.tmx"),
        },
    ));

    // Instructions
    info!("=== Tile Collider Example ===");
    info!("This example requires a map with tiles that have collision shapes defined.");
    info!("");
    info!("To create a map with tile colliders:");
    info!("1. Open Tiled and create a tileset");
    info!("2. Select a tile in the Tileset Editor");
    info!("3. Open View → Tile Collision Editor (Ctrl+Shift+O)");
    info!("4. Draw collision shapes using Rectangle/Polygon/Polyline tools");
    info!("5. Save the tileset and create a map using these tiles");
    info!("6. Place the .tmx file at: crates/bevy_tiledmap_avian/assets/maps/tile_colliders.tmx");
    info!("");
    info!("The plugin will automatically generate optimized colliders!");
    info!("- Rectangular tiles are merged into larger shapes");
    info!("- Custom polygon shapes are preserved");
    info!("- Debug rendering shows all generated colliders (green outlines)");
}
