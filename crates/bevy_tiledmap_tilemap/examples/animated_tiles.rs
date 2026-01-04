//! Animated tiles example.
//!
//! Demonstrates:
//! - Tile animations from Tiled's animation system
//! - Global animation speed control
//! - Pausing/resuming animations
//!
//! Run with: `cargo run --example animated_tiles --features animations`

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
        .add_systems(Update, (camera_movement, animation_controls))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // Load map with animated tiles (water, lava, etc.)
    commands.spawn(TiledMap {
        handle: asset_server.load("maps/animated.tmx"),
    });

    info!("Animated tiles example loaded!");
    info!("WASD - Move camera");
    info!("SPACE - Pause/resume animations");
    info!("UP/DOWN - Increase/decrease animation speed");
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

#[cfg(feature = "animations")]
fn animation_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    paused: Option<Res<AnimationsPaused>>,
    mut speed: ResMut<AnimationSpeed>,
) {
    // Toggle pause
    if keyboard.just_pressed(KeyCode::Space) {
        if paused.is_some() {
            commands.remove_resource::<AnimationsPaused>();
            info!("Animations resumed");
        } else {
            commands.insert_resource(AnimationsPaused);
            info!("Animations paused");
        }
    }

    // Adjust speed
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        speed.0 = (speed.0 * 1.5).min(5.0);
        info!("Animation speed: {:.2}x", speed.0);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        speed.0 = (speed.0 / 1.5).max(0.1);
        info!("Animation speed: {:.2}x", speed.0);
    }
}

#[cfg(not(feature = "animations"))]
fn animation_controls() {
    // No-op when animations feature is disabled
}
