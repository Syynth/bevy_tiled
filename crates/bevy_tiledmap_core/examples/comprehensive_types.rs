//! Generate comprehensive type definitions for the demo project.
//!
//! Run this to generate `tiled_types.json` with all custom property types:
//! ```bash
//! cargo run -p bevy_tiledmap_core --example comprehensive_types
//! ```

use bevy::{asset::AssetPlugin, log::LogPlugin, prelude::*};
use bevy_tiledmap_assets::TiledmapAssetsPlugin;
use bevy_tiledmap_core::prelude::*;

fn main() {
    // Build the app (which exports types during plugin initialization)
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, LogPlugin::default(), AssetPlugin::default()))
        .add_plugins(TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::new(TiledmapCoreConfig {
            // Export to current directory
            export_target: Some(TypeExportTarget::JsonFile("tiled_types.json".into())),
            ..default()
        }));

    // Run the app to trigger the Startup system that exports types
    app.update();
    info!("✅ Type definitions exported to tiled_types.json");
    info!("   (workspace root: assets/tiled_types.json)");
    info!("Import this file into Tiled for autocomplete on custom properties.");
    info!("Next steps:");
    info!("1. Open Tiled");
    info!("2. Go to Edit > Preferences > Custom Types");
    info!("3. Import assets/tiled_types.json");
    info!("4. Start creating maps with autocomplete for custom properties!");
}

// ============================================================================
// Custom TiledClass Definitions
// ============================================================================

/// Gameplay configuration settings.
#[derive(Component, Reflect, TiledClass, Debug, Clone, Default)]
#[reflect(Component)]
#[tiled(name = "GameplaySettings")]
pub struct GameplaySettings {
    /// Player movement speed in pixels per second
    #[tiled(default = 200.0)]
    pub player_speed: f32,

    /// Jump height in pixels
    #[tiled(default = 100.0)]
    pub jump_height: f32,

    /// Allow double jumping
    #[tiled(default = false)]
    pub enable_double_jump: bool,

    /// Gravity multiplier for this area
    #[tiled(default = 1.0)]
    pub gravity_scale: f32,
}

/// Loot configuration for treasure chests and pickups.
#[derive(Component, Reflect, TiledClass, Debug, Clone, Default)]
#[reflect(Component)]
#[tiled(name = "LootSettings")]
pub struct LootSettings {
    /// Rarity tier of the loot
    #[tiled(default = LootTier::Common)]
    pub loot_tier: LootTier,

    /// Number of items to drop
    #[tiled(default = 1)]
    pub quantity: i32,

    /// Time in seconds before loot respawns (0 = no respawn)
    #[tiled(default = 0.0)]
    pub respawn_time: f32,

    /// Gold value multiplier
    #[tiled(default = 1.0)]
    pub value_multiplier: f32,
}

/// Loot rarity tiers.
#[derive(Reflect, TiledClass, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[tiled(name = "LootTier")]
pub enum LootTier {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Enemy AI configuration.
#[derive(Component, Reflect, TiledClass, Debug, Clone, Default)]
#[reflect(Component)]
#[tiled(name = "EnemySettings")]
pub struct EnemySettings {
    /// Type of enemy (determines behavior)
    #[tiled(default = EnemyType::Melee)]
    pub enemy_type: EnemyType,

    /// Patrol movement radius in pixels
    #[tiled(default = 100.0)]
    pub patrol_radius: f32,

    /// Aggro detection range in pixels
    #[tiled(default = 150.0)]
    pub aggro_range: f32,

    /// Enemy health points
    #[tiled(default = 10)]
    pub health: i32,

    /// Movement speed in pixels per second
    #[tiled(default = 50.0)]
    pub move_speed: f32,

    /// Attack damage
    #[tiled(default = 1)]
    pub attack_damage: i32,
}

/// Enemy type variants.
#[derive(Reflect, TiledClass, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[tiled(name = "EnemyType")]
pub enum EnemyType {
    #[default]
    Melee,
    Ranged,
    Flying,
    Boss,
}

/// Spawn point configuration.
#[derive(Component, Reflect, TiledClass, Debug, Clone, Default)]
#[reflect(Component)]
#[tiled(name = "SpawnSettings")]
pub struct SpawnSettings {
    /// Spawn group identifier (e.g., `"player"`, `"enemy_grunt"`, `"item_health"`)
    #[tiled(default = String::new())]
    pub spawn_group: String,

    /// If true, spawn position is randomized within a radius
    #[tiled(default = false)]
    pub is_random_spawn: bool,

    /// Random spawn radius in pixels (if `is_random_spawn` is true)
    #[tiled(default = 0.0)]
    pub random_radius: f32,

    /// Maximum number of entities this spawn point can create (0 = unlimited)
    #[tiled(default = 0)]
    pub max_spawns: i32,
}

/// Waypoint configuration - tests transitive reflection discovery of glam types.
#[derive(Component, Reflect, TiledClass, Debug, Clone, Default)]
#[reflect(Component)]
#[tiled(name = "Waypoint")]
pub struct Waypoint {
    /// Target position (tests `glam::IVec2` discovery via reflection)
    #[tiled(default)]
    pub target_tile: IVec2,

    /// Movement speed to next waypoint
    #[tiled(default = 100.0)]
    pub speed: f32,

    /// Optional offset (tests `glam::Vec2` discovery via reflection)
    pub offset: Option<Vec2>,
}

/// Entity with sprite - tests Handle<Image> field support.
#[derive(Component, Reflect, TiledClass, Debug, Clone, Default)]
#[reflect(Component)]
#[tiled(name = "SpriteEntity")]
pub struct SpriteEntity {
    /// The sprite image asset path
    pub sprite: Option<Handle<Image>>,

    /// Scale factor for the sprite
    #[tiled(default = 1.0)]
    pub scale: f32,
}
