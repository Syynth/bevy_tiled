//! Demonstrates complex enum support with struct and tuple variants.
//!
//! This example shows:
//! 1. Struct variant enums with named fields
//! 2. Tuple variant enums with positional fields
//! 3. Mixed enums (unit + struct variants)
//! 4. Enums with `#[default]` attribute
//! 5. Export to Tiled with `:variant` discriminant field

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

    info!("✅ Complex enum demo completed!");
    info!("Check the generated complex-enum-types.json file");
}

/// Example 1: Struct variant enum (attack abilities)
///
/// This demonstrates named fields in enum variants.
/// Each variant has different fields representing different attack types.
#[derive(Component, Reflect, TiledClass, Clone, Debug, Default)]
#[tiled(name = "demo::Attack")]
pub enum Attack {
    /// No attack
    #[default]
    None,
    /// Melee attack with damage
    Melee { damage: i32 },
    /// Projectile attack with speed and damage
    Projectile {
        speed: f32,
        damage: i32,
        piercing: bool,
    },
}

/// Example 2: Tuple variant enum (movement)
///
/// This demonstrates positional fields in enum variants.
/// Tuple variants use integer field names: "0", "1", etc.
#[derive(Component, Reflect, TiledClass, Clone, Debug, Default)]
#[tiled(name = "demo::Movement")]
pub enum Movement {
    #[default]
    Idle,
    Walk(f32),       // speed
    Dash(Vec2, f32), // direction, speed
}

/// Example 3: Complex enum with nested types
///
/// This shows enum variants containing other custom types.
#[derive(Component, Reflect, TiledClass, Clone, Debug, Default)]
#[tiled(name = "demo::Ability")]
pub enum Ability {
    #[default]
    None,
    Attack {
        attack: Attack,
        cooldown: f32,
    },
    Buff {
        duration: f32,
        strength: f32,
    },
}

/// Example 4: Simple unit-variant enum (for comparison)
///
/// This demonstrates backward compatibility with simple enums.
#[derive(Component, Reflect, TiledClass, Clone, Debug, Default)]
#[tiled(name = "demo::Direction")]
pub enum Direction {
    #[default]
    North,
    South,
    East,
    West,
}

/// Example entity using complex enums
#[derive(Component, Reflect, TiledClass, Default)]
#[tiled(name = "demo::Enemy")]
struct Enemy {
    /// Primary attack ability
    attack: Attack,
    /// Movement pattern
    movement: Movement,
    /// Special ability
    ability: Ability,
    /// Facing direction
    facing: Direction,
    /// Health points
    health: f32,
}

fn export_types(app: &App) {
    info!("🔧 Exporting type definitions...");

    // Export using the hybrid approach (TiledClass + Reflection)
    match bevy_tiledmap_core::properties::export::export_all_types_with_reflection(
        app.world(),
        "complex-enum-types.json",
    ) {
        Ok(()) => {
            info!("✅ Successfully exported types to complex-enum-types.json");
            info!("\n📝 Exported types:");
            info!("  - demo::Attack (complex enum with struct variants)");
            info!("    → Exported as class with :variant field");
            info!("    → Synthetic enum: demo::Attack:::variant");
            info!("  - demo::Movement (complex enum with tuple variants)");
            info!("    → Exported as class with :variant field");
            info!("    → Synthetic enum: demo::Movement:::variant");
            info!("  - demo::Ability (complex enum with nested types)");
            info!("    → Exported as class with :variant field");
            info!("    → Synthetic enum: demo::Ability:::variant");
            info!("  - demo::Direction (simple enum for comparison)");
            info!("    → Exported as native Tiled enum");
            info!("  - demo::Enemy (struct using complex enums)");

            info!("\n💡 Complex enum features:");
            info!("  ✨ Struct variants with named fields");
            info!("  ✨ Tuple variants with positional fields (0, 1, 2, ...)");
            info!("  ✨ Mixed unit + complex variants");
            info!("  ✨ Nested enum types");
            info!("  ✨ Default variant support (#[default])");
            info!("  ✨ Zero boilerplate - derive macro handles everything");

            info!("\n💡 Export format:");
            info!("  - Complex enums → class type with :variant discriminant");
            info!("  - Synthetic enum → dropdown for variant selection");
            info!("  - Field union → all variant fields in class");

            info!("\n💡 To use in Tiled:");
            info!("  1. Open Tiled editor");
            info!("  2. View → Custom Types → Import Custom Types");
            info!("  3. Select complex-enum-types.json");
            info!("  4. Create an object with custom class 'demo::Enemy'");
            info!("  5. Notice complex enum fields have :variant dropdown!");
        }
        Err(e) => {
            error!("❌ Failed to export types: {}", e);
        }
    }
}
