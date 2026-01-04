//! Example demonstrating `bevy_tiledmap` with Avian2D physics integration.
//!
//! This example shows how to use the unified plugin with the `avian` feature
//! to automatically generate physics colliders from Tiled objects and tiles.
//!
//! To run this example, ensure you have the `avian` feature enabled:
//! ```toml
//! bevy_tiledmap = { version = "0.1", features = ["avian"] }
//! ```

use bevy::prelude::*;

#[cfg(feature = "avian")]
use avian2d::prelude::*;

#[cfg(feature = "avian")]
use bevy_tiledmap::prelude::*;

fn main() {
    #[cfg(not(feature = "avian"))]
    {
        eprintln!("This example requires the 'avian' feature!");
        eprintln!("Run with: cargo run --example with_physics --features avian");
        return;
    }

    #[cfg(feature = "avian")]
    {
        App::new()
            .add_plugins(DefaultPlugins)
            // Add Avian2D physics plugin
            .add_plugins(PhysicsPlugins::default())
            // Add unified bevy_tiledmap plugin (includes physics integration)
            .add_plugins(BevyTiledmapPlugin::default())
            .add_systems(Startup, (setup_camera, spawn_map))
            .run();
    }
}

#[cfg(feature = "avian")]
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(feature = "avian")]
fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load and spawn a Tiled map
    // The plugin automatically:
    // - Loads the map asset
    // - Spawns entities for map, layers, and objects
    // - Renders tiles with bevy_ecs_tilemap
    // - Generates physics colliders from objects with `physics_settings` property
    // - Generates tile colliders from tileset collision shapes (if enabled)
    commands.spawn(TiledMap {
        handle: asset_server.load("map.tmx"),
    });

    info!("Map loaded with physics integration!");
    info!("Objects with 'physics_settings' property will have colliders automatically.");
}
