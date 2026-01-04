//! Demonstrates TiledClass type export with Vec2/Vec3 fields and hybrid type resolution.
//!
//! This example shows:
//! 1. Defining TiledClass types with vector fields
//! 2. Exporting type definitions to JSON for Tiled editor
//! 3. How Vec2/Vec3 are properly exported as class types with propertyType

use bevy::prelude::*;
use bevy_tiled_core::plugin::BevyTiledCorePlugin;
use bevy_tiled_macros::TiledClass;

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());

    // Add BevyTiledCorePlugin which registers the TiledClassRegistry
    app.add_plugins(BevyTiledCorePlugin::default());

    // Register glam types in the reflection registry
    app.register_type::<Vec2>();
    app.register_type::<Vec3>();

    // Initialize the app to build registries
    app.finish();
    app.cleanup();

    // Export types directly (not in a system since we need access to &App)
    export_types(&app);

    println!("✅ Type export demo completed!");
    println!("Check the generated custom-types.json file");
}

/// Example TiledClass with Vec2 field
#[derive(Component, Reflect, TiledClass, Default)]
#[tiled(name = "demo::SpawnPoint")]
struct SpawnPoint {
    /// Position in world space
    position: Vec2,
    /// Whether this spawn point is active
    active: bool,
}

/// Example TiledClass with Vec3 field and reference to another TiledClass
#[derive(Component, Reflect, TiledClass, Default)]
#[tiled(name = "demo::Teleporter")]
struct Teleporter {
    /// 3D position of the teleporter
    position: Vec3,
    /// Destination spawn point (would be a reference in real usage)
    destination_id: i32,
    /// Teleporter color
    #[tiled(default = Color::srgba(0.0, 1.0, 1.0, 1.0))]
    color: Color,
}

/// Example with nested position data
#[derive(Component, Reflect, TiledClass, Default)]
#[tiled(name = "demo::Enemy")]
struct Enemy {
    /// Spawn position
    spawn_pos: Vec2,
    /// Health points
    health: f32,
    /// Movement speed
    speed: f32,
}

fn export_types(app: &App) {
    println!("🔧 Exporting type definitions...");

    // Export using the hybrid approach (TiledClass + Reflection)
    match bevy_tiled_core::properties::export::export_all_types_with_reflection(
        app,
        "custom-types.json",
    ) {
        Ok(()) => {
            println!("✅ Successfully exported types to custom-types.json");
            println!("\n📝 Exported types:");
            println!("  - demo::SpawnPoint (with Vec2 position)");
            println!("  - demo::Teleporter (with Vec3 position)");
            println!("  - demo::Enemy (with Vec2 spawn_pos)");
            println!("  - glam::Vec2 (auto-discovered via reflection)");
            println!("  - glam::Vec3 (auto-discovered via reflection)");

            println!("\n💡 To use in Tiled:");
            println!("  1. Open Tiled editor");
            println!("  2. View → Custom Types → Import Custom Types");
            println!("  3. Select custom-types.json");
            println!("  4. Your types will appear in property dropdowns!");
        }
        Err(e) => {
            eprintln!("❌ Failed to export types: {}", e);
        }
    }
}
