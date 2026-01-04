//! Demonstrates `TiledClass` enum support with native Tiled enum export.
//!
//! This example shows:
//! 1. Defining simple unit-variant enums with `#[derive(TiledClass)]`
//! 2. Using enums as fields in `TiledClass` structs
//! 3. Automatic export to Tiled's native enum format (dropdown in editor)
//! 4. Zero-boilerplate `FromTiledProperty` implementation

use bevy::prelude::*;
use bevy_tiledmap_core::plugin::TiledmapCorePlugin;
use bevy_tiledmap_macros::TiledClass;

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::log::LogPlugin::default());

    // Add TiledmapCorePlugin which registers the TiledClassRegistry
    app.add_plugins(TiledmapCorePlugin::default());

    // Initialize the app to build registries
    app.finish();
    app.cleanup();

    // Export types directly
    export_types(&app);

    info!("✅ Enum demo completed!");
    info!("Check the generated custom-types.json file");
}

/// Example enum: Cardinal directions
///
/// This is a unit-variant enum that will be exported as Tiled's native enum type.
/// In Tiled editor, this will appear as a dropdown menu!
#[derive(Component, Reflect, TiledClass, Clone, Debug, Default)]
#[tiled(name = "demo::Direction")]
pub enum Direction {
    #[default]
    North,
    South,
    East,
    West,
}

/// Example enum: Enemy types
#[derive(Component, Reflect, TiledClass, Clone, Debug, Default)]
#[tiled(name = "demo::EnemyType")]
pub enum EnemyType {
    #[default]
    Goblin,
    Orc,
    Troll,
    Dragon,
}

/// Example struct using enums as fields
///
/// This demonstrates how enums are used in `TiledClass` structs.
/// The enum fields will be exported with propertyType references.
#[derive(Component, Reflect, TiledClass, Default)]
#[tiled(name = "demo::Enemy")]
struct Enemy {
    /// Enemy type (uses `EnemyType` enum)
    #[tiled(default = EnemyType::Goblin)]
    enemy_type: EnemyType,
    /// Facing direction (uses Direction enum)
    #[tiled(default = Direction::North)]
    facing: Direction,
    /// Health points
    health: f32,
    /// Movement speed
    speed: f32,
}

/// Example struct: Spawn point with direction
#[derive(Component, Reflect, TiledClass, Default)]
#[tiled(name = "demo::SpawnPoint")]
struct SpawnPoint {
    /// Position in world space
    position: Vec2,
    /// Initial facing direction
    #[tiled(default = Direction::South)]
    direction: Direction,
    /// Whether this spawn point is active
    active: bool,
}

fn export_types(app: &App) {
    info!("🔧 Exporting type definitions...");

    // Export using the hybrid approach (TiledClass + Reflection)
    match bevy_tiledmap_core::properties::export::export_all_types_with_reflection(
        app,
        "custom-types.json",
    ) {
        Ok(()) => {
            info!("✅ Successfully exported types to custom-types.json");
            info!("\n📝 Exported types:");
            info!("  - demo::Direction (enum with 4 variants)");
            info!("  - demo::EnemyType (enum with 4 variants)");
            info!("  - demo::Enemy (struct with enum fields)");
            info!("  - demo::SpawnPoint (struct with enum field)");
            info!("  - glam::Vec2 (auto-discovered via reflection)");

            info!("\n💡 Enum benefits:");
            info!("  ✨ Zero boilerplate - no manual FromTiledProperty impl needed");
            info!("  ✨ Native Tiled enum type with dropdown UI");
            info!("  ✨ Type-safe - prevents typos in variant names");
            info!("  ✨ Auto-discovered when used as struct fields");

            info!("\n💡 To use in Tiled:");
            info!("  1. Open Tiled editor");
            info!("  2. View → Custom Types → Import Custom Types");
            info!("  3. Select custom-types.json");
            info!("  4. Create an object with custom class 'demo::Enemy'");
            info!("  5. Notice 'enemy_type' and 'facing' fields have DROPDOWN menus!");
        }
        Err(e) => {
            error!("❌ Failed to export types: {}", e);
        }
    }
}
