//! Basic tilemap rendering example.
//!
//! Demonstrates:
//! - Loading a simple Tiled map
//! - Rendering tile layers with `bevy_ecs_tilemap`
//! - Camera controls
//!
//! Run with: `cargo run --example basic_tilemap`

use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tiled_assets::BevyTiledAssetsPlugin;
use bevy_tiled_core::prelude::*;
use bevy_tiled_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(BevyTiledAssetsPlugin)
        .add_plugins(BevyTiledCorePlugin::default())
        .add_plugins(BevyTiledTilemapPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, camera_movement)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Load and spawn the Tiled map
    commands.spawn(TiledMap {
        handle: asset_server.load("simple_map.tmx"),
    });

    info!("Basic tilemap example loaded! Use WASD to move camera.");
}

/// Simple camera movement system
fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };
    let speed = 200.0;

    if keyboard.pressed(KeyCode::KeyW) {
        camera_transform.translation.y += speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        camera_transform.translation.y -= speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        camera_transform.translation.x -= speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        camera_transform.translation.x += speed * time.delta_secs();
    }
}
