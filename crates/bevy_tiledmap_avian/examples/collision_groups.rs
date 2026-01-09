//! Example demonstrating collision group filtering using Avian's `CollisionLayers`.
//!
//! This example shows how to configure which objects can collide with each other
//! using collision groups and masks defined in Tiled.
//!
//! # Collision Layer System
//!
//! Avian uses a two-part system:
//! - **Memberships** (`collision_groups)`: Which groups this object belongs to
//! - **Filters** (`collision_mask)`: Which groups this object can collide with
//!
//! Two objects collide if:
//! - Object A's memberships overlap with Object B's filters, AND
//! - Object B's memberships overlap with Object A's filters
//!
//! # Setting Up in Tiled
//!
//! 1. **Define Custom Types** (see `type_export` example)
//!
//! 2. **Configure Objects with `PhysicsSettings`**:
//!    ```
//!    physics_settings: avian::PhysicsSettings {
//!      collision_groups: "player"
//!      collision_mask: "ground,enemies"
//!    }
//!    ```
//!
//! 3. **Implement `collision_layers_fn`**:
//!    This callback converts the string values to Avian's `CollisionLayers`.
//!
//! # Example Scenarios
//!
//! **Player**:
//! - `collision_groups`: "player"
//! - `collision_mask`: "ground,enemies,collectibles"
//! - Result: Player collides with ground, enemies, and collectibles
//!
//! **Enemy**:
//! - `collision_groups`: "enemies"
//! - `collision_mask`: "ground,player"
//! - Result: Enemy collides with ground and player, but not other enemies
//!
//! **Projectile** (player-fired):
//! - `collision_groups`: "`player_projectile`"
//! - `collision_mask`: "enemies,ground"
//! - Result: Hits enemies and ground, passes through player
//!
//! **One-way Platform**:
//! - `collision_groups`: "ground"
//! - `collision_mask`: "player"
//! - Use sensor + custom logic for one-way behavior

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::*;
use bevy_tiledmap_avian::prelude::*;
use bevy_tiledmap_core::prelude::*;

// Define collision layers as constants using LayerMask
// Each layer is a bit position: 1 << n
const PLAYER: LayerMask = LayerMask(1 << 1);
const GROUND: LayerMask = LayerMask(1 << 2);
const ENEMIES: LayerMask = LayerMask(1 << 3);
const COLLECTIBLES: LayerMask = LayerMask(1 << 4);
const PLAYER_PROJECTILE: LayerMask = LayerMask(1 << 5);
const ENEMY_PROJECTILE: LayerMask = LayerMask(1 << 6);
const ALL: LayerMask = LayerMask(u32::MAX);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        .add_plugins(TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::default())
        .add_plugins(TiledmapAvianPlugin::new(PhysicsConfig {
            // Provide custom collision layer parsing
            collision_layers_fn: parse_collision_layers,
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

/// Convert comma-separated collision group strings to Avian's `CollisionLayers`.
///
/// This function is called by the plugin for each object with a `physics_settings` property.
fn parse_collision_layers(groups_str: &str, mask_str: &str) -> CollisionLayers {
    // Parse collision group memberships
    let mut memberships = LayerMask(0);
    for group in groups_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        memberships = LayerMask(
            memberships.0
                | match group {
                    "player" => PLAYER.0,
                    "ground" => GROUND.0,
                    "enemies" => ENEMIES.0,
                    "collectibles" => COLLECTIBLES.0,
                    "player_projectile" => PLAYER_PROJECTILE.0,
                    "enemy_projectile" => ENEMY_PROJECTILE.0,
                    _ => {
                        warn!("Unknown collision group: '{}'", group);
                        0
                    }
                },
        );
    }

    // Parse collision mask filters
    let mut filters = LayerMask(0);
    for group in mask_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        filters = LayerMask(
            filters.0
                | match group {
                    "player" => PLAYER.0,
                    "ground" => GROUND.0,
                    "enemies" => ENEMIES.0,
                    "collectibles" => COLLECTIBLES.0,
                    "player_projectile" => PLAYER_PROJECTILE.0,
                    "enemy_projectile" => ENEMY_PROJECTILE.0,
                    "all" => ALL.0,
                    _ => {
                        warn!("Unknown collision mask: '{}'", group);
                        0
                    }
                },
        );
    }

    // If no filters specified, default to colliding with everything
    if filters.0 == 0 && !mask_str.is_empty() {
        filters = ALL;
    }

    CollisionLayers::new(memberships, filters)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Load map with objects configured with collision groups
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        TiledMap {
            handle: asset_server.load("maps/basic_physics.tmx"),
        },
    ));

    // Instructions
    info!("=== Collision Groups Example ===");
    info!("");
    info!("This example demonstrates collision filtering with groups.");
    info!("");
    info!("Collision groups defined:");
    info!("- player: Player character");
    info!("- ground: Static terrain");
    info!("- enemies: Enemy characters");
    info!("- collectibles: Items to collect");
    info!("- player_projectile: Player-fired projectiles");
    info!("- enemy_projectile: Enemy-fired projectiles");
    info!("");
    info!("Configure in Tiled using physics_settings:");
    info!("  collision_groups: 'player'");
    info!("  collision_mask: 'ground,enemies,collectibles'");
    info!("");
    info!("Two objects collide if their groups/masks overlap!");
}
