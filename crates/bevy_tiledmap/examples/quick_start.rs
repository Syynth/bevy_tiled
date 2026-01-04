//! Quick start example demonstrating basic `bevy_tiledmap` usage.
//!
//! This example shows the simplest way to load and render a Tiled map using
//! the unified `BevyTiledmapPlugin`.

use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the unified bevy_tiledmap plugin with default settings
        .add_plugins(BevyTiledmapPlugin::default())
        .add_systems(Startup, (setup_camera, spawn_map))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load and spawn a Tiled map
    // The plugin automatically handles:
    // - Asset loading (.tmx file)
    // - Entity spawning (map, layers, objects)
    // - Rendering (with bevy_ecs_tilemap)
    commands.spawn(TiledMap {
        handle: asset_server.load("map.tmx"),
    });

    info!("Map loaded! The unified plugin handles everything automatically.");
}
