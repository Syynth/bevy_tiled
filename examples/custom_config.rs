//! Example demonstrating custom configuration of the unified plugin.
//!
//! This example shows how to customize the behavior of each layer using
//! the builder pattern.

use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Configure the unified plugin with custom settings for each layer
        .add_plugins(
            BevyTiledmapPlugin::default()
                // Layer 2: Core configuration
                .with_core(TiledmapCoreConfig {
                    // Export type definitions for Tiled editor autocomplete
                    export_types_path: Some("assets/tiled_types.json".into()),
                    ..default()
                })
                // Layer 3: Tilemap rendering configuration
                .with_tilemap(TilemapRenderConfig {
                    // Enable tile animations
                    enable_animations: true,
                    // Enable parallax scrolling for layers
                    enable_parallax: true,
                    // Disable debug shape rendering
                    enable_debug_shapes: false,
                }),
        )
        .add_systems(Startup, (setup_camera, spawn_map))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TiledMap {
        handle: asset_server.load("map.tmx"),
    });

    info!("Map loaded with custom configuration!");
    info!("- Type definitions exported to assets/tiled_types.json");
    info!("- Animations enabled");
    info!("- Parallax scrolling enabled");
}
