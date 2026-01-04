//! Complete example showing all layer types and features.
//!
//! Demonstrates:
//! - Tile layers with animations
//! - Object layers (tile objects and shapes)
//! - Image layers
//! - Parallax scrolling
//! - Z-ordering
//! - Debug shape rendering
//!
//! Run with: `cargo run --example all_layers --features animations,parallax,debug_shapes`

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
        .add_systems(Update, (camera_movement, controls, debug_ui))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera with parallax support
    commands.spawn((Camera2d, ParallaxCamera));

    // Load comprehensive map with all layer types
    commands.spawn(TiledMap {
        handle: asset_server.load("maps/complete.tmx"),
    });

    info!("Complete bevy_tiledmap_tilemap example loaded!");
    info!("");
    info!("Controls:");
    info!("  WASD          - Move camera");
    info!("  SPACE         - Pause/resume animations");
    info!("  UP/DOWN       - Animation speed");
    info!("  G             - Toggle debug shapes");
    info!("  +/-           - Zoom");
    info!("");
    info!("Features demonstrated:");
    info!("  ✓ Tile layers with multi-tileset support");
    info!("  ✓ Animated tiles (water, lava, etc.)");
    info!("  ✓ Object layers (enemies, items, collision)");
    info!("  ✓ Image layers (backgrounds)");
    info!("  ✓ Parallax scrolling");
    info!("  ✓ Z-ordering (layers stack correctly)");
    info!("  ✓ Debug shapes (press G)");
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

    // Movement
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

    // Zoom
    if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        camera_transform.scale *= 0.99;
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        camera_transform.scale *= 1.01;
    }
}

fn controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    #[cfg(feature = "debug_shapes")] mut config: ResMut<TilemapRenderConfig>,
    #[cfg(feature = "animations")] paused: Option<Res<AnimationsPaused>>,
    #[cfg(feature = "animations")] mut speed: ResMut<AnimationSpeed>,
) {
    // Toggle animations
    #[cfg(feature = "animations")]
    if keyboard.just_pressed(KeyCode::Space) {
        if paused.is_some() {
            commands.remove_resource::<AnimationsPaused>();
            info!("✓ Animations resumed");
        } else {
            commands.insert_resource(AnimationsPaused);
            info!("⏸ Animations paused");
        }
    }

    // Animation speed
    #[cfg(feature = "animations")]
    {
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            speed.0 = (speed.0 * 1.5).min(5.0);
            info!("Animation speed: {:.2}x", speed.0);
        }
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            speed.0 = (speed.0 / 1.5).max(0.1);
            info!("Animation speed: {:.2}x", speed.0);
        }
    }

    // Toggle debug shapes
    #[cfg(feature = "debug_shapes")]
    if keyboard.just_pressed(KeyCode::KeyG) {
        config.enable_debug_shapes = !config.enable_debug_shapes;
        if config.enable_debug_shapes {
            info!("✓ Debug shapes enabled");
        } else {
            info!("✗ Debug shapes disabled");
        }
    }
}

fn debug_ui() {
    // Placeholder for future UI rendering
    // Could add egui or bevy_ui panels here to show status
}
