//! Comprehensive demo showing all `bevy_tiledmap` features.
//!
//! Features:
//! - Tiled map loading with multiple layers
//! - Tilemap rendering via `bevy_ecs_tilemap`
//! - `Avian2D` physics integration with colliders
//! - Player movement with WASD controls
//! - Inspector UI for debugging (press ESC to toggle)
//!
//! Run with:
//! ```bash
//! cargo run --example demo
//! ```

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tiledmap::prelude::*;
use bevy_tiledmap_core::components::map::MapGeometry;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add Avian2D physics with debug rendering
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        // Add bevy_tiledmap with type export
        .add_plugins(BevyTiledmapPlugin {
            core: TiledmapCoreConfig {
                export_types_path: Some("assets/tiled_types.json".into()),
            },
            ..default()
        })
        // Add inspector for debugging
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, (setup_camera, spawn_map))
        .add_systems(Update, (player_movement, fit_camera_to_map, draw_map_bounds_gizmos))
        .run();
}

/// Marker component for the player entity
#[derive(Component, Reflect, TiledClass)]
#[tiled(name = "Player")]
#[reflect(Component)]
struct Player;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the Tiled world
    // The plugin automatically:
    // - Loads all maps in the world
    // - Loads all tilesets and images
    // - Spawns entities for maps, layers, and objects
    // - Renders tiles with bevy_ecs_tilemap
    // - Generates physics colliders from objects with avian::PhysicsSettings
    // - Processes custom properties via TiledClass
    commands.spawn(TiledWorld {
        handle: asset_server.load("demo.world"),
    });

    info!("🗺️  World loading...");
    info!("🎮 Controls: WASD to move");
    info!("🔍 Press ESC to toggle inspector");
}

/// Handle WASD input and set player velocity
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
) {
    for mut velocity in &mut player_query {
        let mut direction = Vec2::ZERO;

        // Read WASD input
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Normalize diagonal movement and apply speed
        if direction != Vec2::ZERO {
            direction = direction.normalize();
        }

        const PLAYER_SPEED: f32 = 200.0;
        velocity.0 = direction * PLAYER_SPEED;
    }
}

/// Fit camera to show the entire map
fn fit_camera_to_map(
    map_query: Query<&MapGeometry, Added<MapGeometry>>,
    mut camera_query: Query<(&mut Transform, &mut Camera), With<Camera2d>>,
    windows: Query<&Window>,
) {
    for geometry in &map_query {
        let Ok((mut transform, mut camera)) = camera_query.single_mut() else {
            return;
        };
        let Ok(window) = windows.single() else {
            return;
        };

        // Center camera on map
        let center = geometry.bounds.center();
        transform.translation = Vec3::new(center.x, center.y, transform.translation.z);

        // Scale to fit map in window
        let map_size = geometry.bounds.size();
        let window_size = Vec2::new(window.width(), window.height());
        let scale_x = map_size.x / window_size.x;
        let scale_y = map_size.y / window_size.y;
        let scale = scale_x.max(scale_y) * 1.1; // 10% padding

        camera.orthographic_mut().unwrap().scale = scale;
    }
}

/// Draw gizmos at map corners to help debug coordinate alignment
fn draw_map_bounds_gizmos(map_query: Query<&MapGeometry>, mut gizmos: Gizmos) {
    for geometry in &map_query {
        let bounds = geometry.bounds;

        // Draw circles at corners
        // GREEN = origin (0, 0) - should be bottom-left
        gizmos.circle_2d(bounds.min, 8.0, Color::srgb(0.0, 1.0, 0.0));

        // RED = max (width, height) - should be top-right
        gizmos.circle_2d(bounds.max, 8.0, Color::srgb(1.0, 0.0, 0.0));

        // BLUE = top-left corner
        gizmos.circle_2d(Vec2::new(bounds.min.x, bounds.max.y), 8.0, Color::srgb(0.0, 0.0, 1.0));

        // YELLOW = bottom-right corner
        gizmos.circle_2d(Vec2::new(bounds.max.x, bounds.min.y), 8.0, Color::srgb(1.0, 1.0, 0.0));

        // Draw bounding rectangle
        gizmos.rect_2d(bounds.center(), bounds.size(), Color::srgb(1.0, 1.0, 1.0));
    }
}
