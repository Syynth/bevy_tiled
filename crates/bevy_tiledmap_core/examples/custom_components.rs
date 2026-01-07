//! Example demonstrating the full property-to-component workflow.
//!
//! This example shows:
//! - Defining custom components with `#[derive(TiledClass)]`
//! - Automatic component attachment during map spawning
//! - Querying entities with auto-attached components
//! - Using observers to react to `ObjectSpawned` events
//! - Accessing raw properties via `MergedProperties`
//!
//! # Setup
//!
//! 1. Run this example: `cargo run --example custom_components`
//! 2. The example will print information about spawned objects and their components
//! 3. Press Ctrl+C to exit

use bevy::prelude::*;
use bevy_tiledmap_assets::TiledmapAssetsPlugin;
use bevy_tiledmap_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::default())
        // Register our custom components for reflection
        .register_type::<Player>()
        .register_type::<Enemy>()
        .register_type::<Collectible>()
        .register_type::<SpawnPoint>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (demonstrate_component_queries, demonstrate_merged_properties),
        )
        // Observe ObjectSpawned events
        .add_observer(on_object_spawned)
        .run();
}

/// Spawn a map with custom component properties
fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    info!("=== Custom Components Example ===");
    info!("This example demonstrates the property-to-component system.");
    info!("");

    // Spawn a camera
    commands.spawn(Camera2d);

    // TODO: Load a test map with objects that have custom properties
    // For now, we'll manually create some test objects to demonstrate the system

    info!("NOTE: This example needs a Tiled map asset to fully demonstrate.");
    info!("The component definitions below show what would be auto-attached.");
    info!("");
}

/// Demonstrate querying for auto-attached components
fn demonstrate_component_queries(
    player_query: Query<(Entity, &Player, &Transform), Added<Player>>,
    enemy_query: Query<(Entity, &Enemy, &Transform), Added<Enemy>>,
    collectible_query: Query<(Entity, &Collectible), Added<Collectible>>,
) {
    // Query for newly spawned players
    for (entity, player, transform) in player_query.iter() {
        info!("🎮 Player spawned!");
        info!("  Entity: {:?}", entity);
        info!("  Health: {}", player.health);
        info!("  Speed: {}", player.speed);
        info!("  Position: {:?}", transform.translation);
        info!("");
    }

    // Query for newly spawned enemies
    for (entity, enemy, transform) in enemy_query.iter() {
        info!("👾 Enemy spawned!");
        info!("  Entity: {:?}", entity);
        info!("  Type: {:?}", enemy.enemy_type);
        info!("  Damage: {}", enemy.damage);
        info!("  Patrol speed: {}", enemy.patrol_speed);
        info!("  Position: {:?}", transform.translation);
        info!("");
    }

    // Query for newly spawned collectibles
    for (entity, collectible) in collectible_query.iter() {
        info!("💎 Collectible spawned!");
        info!("  Entity: {:?}", entity);
        info!("  Points: {}", collectible.points);
        info!("  Respawns: {}", collectible.respawns);
        info!("");
    }
}

/// Demonstrate accessing raw properties via `MergedProperties`
fn demonstrate_merged_properties(
    query: Query<(Entity, &MergedProperties, &TiledObject), Added<MergedProperties>>,
) {
    for (entity, props, object_type) in query.iter() {
        info!("📦 Object with properties spawned!");
        info!("  Entity: {:?}", entity);
        info!("  Type: {:?}", object_type);
        info!("  Properties:");

        // Access properties by name
        if let Some(value) = props.get_bool("enabled") {
            info!("    enabled: {}", value);
        }

        if let Some(value) = props.get_i32("priority") {
            info!("    priority: {}", value);
        }

        if let Some(value) = props.get_string("description") {
            info!("    description: {}", value);
        }

        // Iterate all properties
        info!("  All properties:");
        for (key, value) in props.iter() {
            info!("    {}: {:?}", key, value);
        }
        info!("");
    }
}

/// Observer that reacts to `ObjectSpawned` events
fn on_object_spawned(trigger: On<ObjectSpawned>, objects: Query<&TiledObject>) {
    let event = trigger.event();

    info!("🔔 ObjectSpawned event triggered!");
    info!("  Entity: {:?}", event.entity);
    info!("  Map: {:?}", event.map_entity);
    info!("  Object ID: {}", event.object_id);

    // Access the object component
    if let Ok(object) = objects.get(event.entity) {
        info!("  Object type: {:?}", object);
    }

    // Access properties from the event
    info!("  Properties in event: {} entries", event.properties.len());
    info!("");
}

// ============================================================================
// Custom Component Definitions
// ============================================================================

/// Player component - automatically attached to objects with type `"game::Player"`
#[derive(Component, Reflect, Default, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "game::Player")]
pub struct Player {
    /// Player health points
    #[tiled(default = 100.0)]
    pub health: f32,

    /// Movement speed
    #[tiled(default = 5.0)]
    pub speed: f32,

    /// Player name (optional)
    pub name: Option<String>,

    /// Starting inventory item count
    #[tiled(default = 0)]
    pub inventory_size: i32,
}

/// Enemy component - automatically attached to objects with type `"game::Enemy"`
#[derive(Component, Reflect, Default, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "game::Enemy")]
pub struct Enemy {
    /// Type of enemy
    pub enemy_type: EnemyType,

    /// Damage dealt per hit
    #[tiled(default = 10)]
    pub damage: i32,

    /// Patrol movement speed
    #[tiled(default = 2.0)]
    pub patrol_speed: f32,

    /// Detection range
    #[tiled(default = 100.0)]
    pub detection_range: f32,

    /// Enemy color tint
    #[tiled(default = Color::srgb(1.0, 0.0, 0.0))]
    pub color: Color,
}

/// Enemy type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum EnemyType {
    #[default]
    Grunt,
    Elite,
    Boss,
}

// Implement FromTiledProperty for EnemyType to deserialize from string
impl FromTiledProperty for EnemyType {
    fn from_property(value: &tiled::PropertyValue) -> Option<Self> {
        match value {
            tiled::PropertyValue::StringValue(s) => match s.as_str() {
                "Grunt" => Some(EnemyType::Grunt),
                "Elite" => Some(EnemyType::Elite),
                "Boss" => Some(EnemyType::Boss),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Collectible component - for items that can be picked up
#[derive(Component, Reflect, Default, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "game::Collectible")]
pub struct Collectible {
    /// Points awarded when collected
    #[tiled(default = 10)]
    pub points: i32,

    /// Item name
    pub name: String,

    /// Whether the item respawns after collection
    #[tiled(default = false)]
    pub respawns: bool,

    /// Respawn delay in seconds (only used if respawns = true)
    #[tiled(default = 30.0)]
    pub respawn_delay: f32,
}

/// Spawn point component - marks locations where entities spawn
#[derive(Component, Reflect, Default, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "game::SpawnPoint")]
pub struct SpawnPoint {
    /// What type of entity spawns here
    pub spawn_type: String,

    /// Spawn delay in seconds
    #[tiled(default = 0.0)]
    pub delay: f32,

    /// Whether this spawn point is active
    #[tiled(default = true)]
    pub active: bool,

    /// Team/faction ID
    #[tiled(default = 0)]
    pub team_id: i32,
}
