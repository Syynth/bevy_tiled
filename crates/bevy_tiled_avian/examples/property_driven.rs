//! Example demonstrating property-driven physics configuration using `PhysicsSettings`.
//!
//! This example shows how to configure physics parameters for individual objects
//! directly in Tiled using the `PhysicsSettings` custom class.
//!
//! # Setting Up in Tiled
//!
//! 1. **Export Custom Types** (one-time setup):
//!    - Run the `type_export` example to generate `physics-types.json`
//!    - In Tiled: View → Custom Types Editor → Import → Select `physics-types.json`
//!
//! 2. **Configure Objects**:
//!    - Create an object in your map
//!    - In the Properties panel, click "+" to add a custom property
//!    - Name: `physics_settings`
//!    - Type: `avian::PhysicsSettings`
//!    - Configure the nested properties:
//!      - `body_type`: "Static", "Dynamic", or "Kinematic"
//!      - `friction`: 0.0 (ice) to 1.0+ (sticky)
//!      - `restitution`: 0.0 (no bounce) to 1.0 (perfect bounce)
//!      - `density`: Mass per unit area (kg/m²)
//!      - `is_sensor`: true for triggers that don't generate collision responses
//!      - `lock_rotation`: true to prevent rotation
//!      - `linear_damping`: Reduces linear velocity over time
//!      - `angular_damping`: Reduces angular velocity over time
//!      - `gravity_scale`: Multiplier for gravity (0.0 = no gravity)
//!
//! # Example Configurations
//!
//! **Bouncy Ball** (Dynamic):
//! - `body_type`: "Dynamic"
//! - friction: 0.1
//! - restitution: 0.9
//! - density: 1.0
//!
//! **Ice Platform** (Static):
//! - `body_type`: "Static"
//! - friction: 0.05
//! - restitution: 0.0
//!
//! **Trigger Zone** (Sensor):
//! - `body_type`: "Static"
//! - `is_sensor`: true
//!
//! **Moving Platform** (Kinematic):
//! - `body_type`: "Kinematic"
//! - friction: 0.8

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiled_avian::prelude::*;
use bevy_tiled_assets::prelude::*;
use bevy_tiled_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin) // Visualize colliders
        .add_plugins(BevyTiledAssetsPlugin)
        .add_plugins(BevyTiledCorePlugin::default())
        .add_plugins(BevyTiledAvianPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Load map with objects configured via PhysicsSettings
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        TiledMap {
            handle: asset_server.load("maps/basic_physics.tmx"),
        },
    ));

    // Instructions
    info!("=== Property-Driven Physics Example ===");
    info!("");
    info!("This example demonstrates configuring physics via Tiled properties.");
    info!("");
    info!("The loaded map should have objects with 'physics_settings' properties:");
    info!("- Static platforms with varying friction");
    info!("- Dynamic objects with different restitution (bounciness)");
    info!("- Kinematic platforms");
    info!("- Sensor triggers");
    info!("");
    info!("Each object's physics is configured entirely in Tiled!");
    info!("Green outlines = collision shapes (from PhysicsDebugPlugin)");
}
