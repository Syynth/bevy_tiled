//! Parallax scrolling example.
//!
//! Demonstrates:
//! - Parallax scrolling for multiple layers
//! - Different parallax factors creating depth
//! - `ParallaxCamera` marker component
//!
//! Run with: `cargo run --example parallax_layers --features parallax`

use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tiledmap_assets::TiledmapAssetsPlugin;
use bevy_tiledmap_core::prelude::*;
use bevy_tiledmap_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::default())
        .add_plugins(TilemapPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, camera_movement)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera with ParallaxCamera marker
    commands.spawn((Camera2d, ParallaxCamera));

    // Load map with parallax layers
    // In Tiled, set custom properties on layers:
    // - parallaxX: 0.5 (slower = appears further away)
    // - parallaxY: 0.5
    commands.spawn(TiledMap {
        handle: asset_server.load("maps/parallax.tmx"),
    });

    info!("Parallax scrolling example loaded!");
    info!("WASD - Move camera (watch the layers scroll at different speeds!)");
    info!("");
    info!("Parallax setup in Tiled:");
    info!("  Background layer: parallaxX=0.3, parallaxY=0.3 (slow)");
    info!("  Middle layer:     parallaxX=0.6, parallaxY=0.6");
    info!("  Foreground layer: parallaxX=1.0, parallaxY=1.0 (normal)");
}

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
