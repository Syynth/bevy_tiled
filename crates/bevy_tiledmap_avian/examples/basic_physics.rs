//! Basic physics example with debug visualization.
//!
//! This example demonstrates:
//! - Loading a Tiled map with objects
//! - Automatic collider generation from object shapes
//! - Debug gizmo visualization of physics colliders
//!
//! # Controls
//!
//! - Use mouse wheel to zoom camera
//! - Colliders are shown with green outlines (Avian debug gizmos)
//!
//! # Setup
//!
//! Create a simple Tiled map with some objects:
//! - Rectangle objects
//! - Ellipse objects
//! - Polygon objects
//! - Point objects
//!
//! All objects will automatically get physics colliders based on their shapes.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::*;
use bevy_tiledmap_avian::prelude::*;
use bevy_tiledmap_core::components::map::TiledMap;
use bevy_tiledmap_core::TiledmapCorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        // Add Avian physics with debug visualization
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        // Add bevy_tiled layers
        .add_plugins(TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::default())
        // Add physics integration (Phase 1: uses global defaults)
        .add_plugins(TiledmapAvianPlugin::default())
        // Setup
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 1000.0)));

    // Load a Tiled map
    let map_handle: Handle<TiledMapAsset> = asset_server.load("maps/basic_physics.tmx");

    commands.spawn(TiledMap { handle: map_handle });

    info!("🎮 Basic Physics Example");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✨ Physics colliders will be created for all objects");
    info!("🟢 Green outlines show collider shapes (Avian debug)");
    info!("📦 All objects use global defaults:");
    info!("   - Static rigid bodies");
    info!("   - Friction: 0.5");
    info!("   - Restitution: 0.0");
    info!("");
    info!("💡 Create a map at assets/maps/basic_physics.tmx with:");
    info!("   - Rectangle objects → Rectangle colliders");
    info!("   - Ellipse objects → Circle colliders");
    info!("   - Polygon objects → Convex hull colliders");
    info!("   - Point objects → Small circle colliders");
}
