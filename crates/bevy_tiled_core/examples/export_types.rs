//! Example demonstrating `TiledClass` type export to JSON.
//!
//! This example shows:
//! - Defining custom components with `#[derive(TiledClass)]`
//! - Configuring the plugin to export type definitions
//! - The exported JSON format for Tiled editor integration
//!
//! Run this example to generate `exported_types.json` in the current directory.
//! This JSON file can be used with Tiled to get autocomplete for custom properties.

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_tiled_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        // Configure plugin to export types on startup
        .add_plugins(BevyTiledCorePlugin::new(BevyTiledCoreConfig {
            export_types_path: Some("exported_types.json".into()),
        }))
        .add_systems(Startup, (print_info, exit_after_export))
        .run();
}

fn print_info() {
    info!("=== TiledClass Export Example ===");
    info!("Types have been exported to: exported_types.json");
    info!("");
    info!("Registered types:");
    info!("  - game::Door (locked: bool, key_id: Option<u32>, health: f32)");
    info!("  - game::Enemy (damage: i32, speed: f32, color: Color)");
    info!("  - game::Item (name: String, position: Vec2)");
    info!("  - game::Trigger (once: bool, enabled: bool, trigger_id: u32)");
    info!("");
    info!("Check exported_types.json to see the format!");
}

fn exit_after_export(mut exit: MessageWriter<AppExit>) {
    // Exit immediately after startup - we only need to export types
    exit.write(AppExit::Success);
}

// Example component: Door with various property types
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Door")]
struct Door {
    /// Whether the door is locked
    locked: bool,

    /// Optional key ID required to unlock
    key_id: Option<u32>,

    /// Door health (with default value)
    #[tiled(default = 100.0)]
    health: f32,

    /// This field is skipped and won't appear in Tiled
    #[tiled(skip)]
    runtime_state: DoorState,
}

#[derive(Default, Reflect)]
enum DoorState {
    #[default]
    Closed,
    Open,
    Locked,
}

// Example component: Enemy with different property types
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Enemy")]
struct Enemy {
    /// Damage dealt per hit
    damage: i32,

    /// Movement speed
    speed: f32,

    /// Enemy color tint
    color: Color,
}

// Example component: Item with string and vector properties
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Item")]
struct Item {
    /// Item name
    name: String,

    /// Spawn position offset (stored as "x,y" string in Tiled)
    position: Vec2,

    /// Optional item category
    category: Option<String>,
}

// Example component: Trigger zone with boolean flags
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Trigger")]
struct Trigger {
    /// Whether trigger is one-time use
    #[tiled(default = false)]
    once: bool,

    /// Whether trigger is enabled by default
    #[tiled(default = true)]
    enabled: bool,

    /// Trigger ID for scripting
    trigger_id: u32,
}
