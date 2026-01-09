//! Infinite map example demonstrating chunk-based tile layer support.
//!
//! Tiled's "infinite" maps use chunk-based storage (16x16 tile chunks) that can
//! extend in any direction, including negative coordinates. This example shows
//! that infinite maps load and render correctly.
//!
//! Run with:
//! ```bash
//! cargo run --example infinite_map
//! ```

use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(BevyTiledmapPlugin::default())
        .add_systems(Startup, (setup_camera, spawn_map))
        .add_systems(Update, camera_movement)
        .run();
}

fn setup_camera(mut commands: Commands) {
    // Spawn camera - position in center of the map (3x3 chunks of 16x16 tiles at 16px each)
    // Map spans from chunk (-1,-1) to (1,1), so center is around (24*16/2, 24*16/2) = (384, 384)
    commands.spawn((Camera2d, Transform::from_xyz(384.0, 384.0, 0.0)));
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the infinite map
    commands.spawn(TiledMap {
        handle: asset_server.load("infinite_map.tmx"),
    });

    info!("Infinite map example loaded!");
    info!("Controls: WASD to pan camera");
}

/// Camera panning with WASD
fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let speed = 300.0;

    if keyboard.pressed(KeyCode::KeyW) {
        transform.translation.y += speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        transform.translation.x += speed * time.delta_secs();
    }
}
