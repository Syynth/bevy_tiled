//! Export physics types to JSON for Tiled autocomplete.
//!
//! This example demonstrates how to export the `PhysicsSettings` and `BodyType`
//! custom types to a JSON file that can be imported into Tiled editor.
//!
//! # Usage
//!
//! 1. Run this example to generate `physics-types.json`
//! 2. Open Tiled editor
//! 3. View → Custom Types Editor
//! 4. Click "Import Custom Types"
//! 5. Select `physics-types.json`
//! 6. Your types will appear in the property dropdown!
//!
//! # What Gets Exported
//!
//! - `avian::PhysicsSettings` - Comprehensive physics configuration class
//! - `avian::BodyType` - Enum for Static/Dynamic/Kinematic
//!
//! # Example Usage in Tiled
//!
//! After importing:
//! 1. Create an object in your map
//! 2. Add a custom property named `physics_settings`
//! 3. Set type to `avian::PhysicsSettings`
//! 4. Configure the physics parameters with autocomplete!

use bevy::prelude::*;
use bevy_tiledmap_avian::prelude::*;
use bevy_tiledmap_core::prelude::*;

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::log::LogPlugin::default());

    // Add bevy_tiledmap_core (registers TiledClassRegistry)
    app.add_plugins(TiledmapCorePlugin::default());

    // Add bevy_tiledmap_avian (registers PhysicsSettings and BodyType)
    app.add_plugins(TiledmapAvianPlugin::default());

    // Initialize the app to build registries
    app.finish();
    app.cleanup();

    // Export types
    export_types(&app);

    info!("✅ Type export completed!");
    info!("Check the generated physics-types.json file");
}

fn export_types(app: &App) {
    info!("🔧 Exporting physics type definitions...");

    // Export using the hybrid approach (TiledClass + Reflection)
    match bevy_tiledmap_core::properties::export::export_all_types_with_reflection(
        app,
        "physics-types.json",
    ) {
        Ok(()) => {
            info!("✅ Successfully exported types to physics-types.json");
            info!("");
            info!("📝 Exported types:");
            info!("  - avian::PhysicsSettings (comprehensive physics configuration)");
            info!("    Fields:");
            info!("      • body_type: avian::BodyType (Static/Dynamic/Kinematic)");
            info!("      • friction: float (0.0 - 1.0+)");
            info!("      • restitution: float (0.0 - 1.0)");
            info!("      • density: float (kg/m²)");
            info!("      • collision_groups: string (comma-separated)");
            info!("      • collision_mask: string (comma-separated)");
            info!("      • is_sensor: bool");
            info!("      • linear_damping: float (optional)");
            info!("      • angular_damping: float (optional)");
            info!("      • gravity_scale: float (optional)");
            info!("      • lock_rotation: bool");
            info!("");
            info!("  - avian::BodyType (enum)");
            info!("    Variants: Static, Dynamic, Kinematic");
            info!("");
            info!("💡 To use in Tiled:");
            info!("  1. Open Tiled editor");
            info!("  2. View → Custom Types Editor");
            info!("  3. Click 'Import Custom Types'");
            info!("  4. Select physics-types.json");
            info!("  5. Create an object in your map");
            info!("  6. Add property 'physics_settings' with type 'avian::PhysicsSettings'");
            info!("  7. Configure physics parameters with autocomplete!");
            info!("");
            info!("✨ Example property in Tiled:");
            info!("   Property name: physics_settings");
            info!("   Property type: avian::PhysicsSettings");
            info!("   Values:");
            info!("     body_type: Dynamic");
            info!("     friction: 0.8");
            info!("     restitution: 0.3");
            info!("     collision_groups: player");
            info!("     collision_mask: ground,enemies");
        }
        Err(e) => {
            error!("❌ Failed to export types: {}", e);
        }
    }
}
