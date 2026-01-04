//! Layer 3 extension pattern example.
//!
//! This example demonstrates how to build Layer 3 plugins that extend Layer 2
//! with rendering, physics, or custom game logic. Layer 2 (`bevy_tiled_core`)
//! provides the entity structure and data; Layer 3 plugins add visual/behavioral
//! components by observing spawning events.
//!
//! ## The Three-Layer Architecture
//!
//! - **Layer 1 (`bevy_tiled_assets`)**: Asset loading (.tmx, .tsx, .tx files)
//! - **Layer 2 (`bevy_tiled_core`)**: Entity spawning, property merging, events
//! - **Layer 3 (your plugins)**: Rendering, physics, game logic
//!
//! ## This Example Shows
//!
//! 1. **Simple sprite rendering plugin** - adds sprites to tile layers
//! 2. **Physics plugin** - adds colliders to objects based on shape
//! 3. **Custom component plugin** - reacts to spawned `TiledClass` components
//!
//! These are minimal examples - real Layer 3 plugins would use
//! `bevy_ecs_tilemap`, Avian physics, etc.

use bevy::prelude::*;
use bevy_tiled_assets::prelude::*;
use bevy_tiled_core::components::tile::TileLayerData;
use bevy_tiled_core::events::*;
use bevy_tiled_core::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            BevyTiledAssetsPlugin,
            BevyTiledCorePlugin::default(),
        ))
        // Layer 3 plugins - demonstrate extension pattern
        .add_plugins((
            SimpleSpriteRenderPlugin,
            SimplePhysicsPlugin,
            CustomComponentPlugin,
        ))
        .add_systems(Startup, (setup_camera, spawn_map))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("simple_map.tmx");

    commands.spawn((
        TiledMap { handle: map_handle },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Demo Map"),
    ));

    info!("Map spawned - Layer 3 plugins will react to spawning events");
}

// ============================================================================
// Layer 3 Example: Simple Sprite Rendering Plugin
// ============================================================================

/// Minimal rendering plugin that adds sprites to tile layers.
///
/// A real implementation would use `bevy_ecs_tilemap` for performance,
/// but this demonstrates the extension pattern clearly.
pub struct SimpleSpriteRenderPlugin;

impl Plugin for SimpleSpriteRenderPlugin {
    fn build(&self, app: &mut App) {
        // Observe TileLayerSpawned events from Layer 2
        app.add_observer(on_tile_layer_spawned);
    }
}

/// Observer that reacts to tile layer spawning.
///
/// This is the Layer 3 extension point - Layer 2 emits `TileLayerSpawned`,
/// Layer 3 adds rendering components.
fn on_tile_layer_spawned(
    trigger: On<TileLayerSpawned>,
    layer_query: Query<(&TileLayerData, &Name)>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let Ok((tile_data, layer_name)) = layer_query.get(event.entity) else {
        return;
    };

    info!(
        "🎨 SimpleSpriteRenderPlugin: Tile layer '{}' spawned with {}x{} tiles",
        layer_name, tile_data.width, tile_data.height
    );

    // In a real plugin, you'd create a tilemap here
    // For this example, we just spawn a placeholder sprite per tile

    let tile_count: usize = tile_data.tiles.iter().filter(|t| t.is_some()).count();

    // Spawn child entities for each tile (simplified - real plugins use batching)
    for (x, y, tile) in tile_data.iter_tiles().take(10) {
        // Only show first 10 for demo
        let Some(_tileset) = tileset_assets.get(&tile.tileset_handle) else {
            continue;
        };

        // Calculate world position
        let tile_size = 16.0; // Would come from tileset in real implementation
        let world_x = x as f32 * tile_size;
        let world_y = -(y as f32 * tile_size); // Tiled Y is down, Bevy Y is up

        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                custom_size: Some(Vec2::splat(tile_size)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, 0.0),
            Name::new(format!("Tile ({}, {})", x, y)),
        ));
    }

    info!(
        "  ✓ Rendered {} tiles (showing first 10)",
        tile_count.min(10)
    );
}

// ============================================================================
// Layer 3 Example: Simple Physics Plugin
// ============================================================================

/// Minimal physics plugin that adds placeholder colliders to objects.
///
/// A real implementation would use Avian or Rapier and actually generate
/// proper collision shapes, but this shows the pattern.
pub struct SimplePhysicsPlugin;

impl Plugin for SimplePhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Observe ObjectSpawned events from Layer 2
        app.add_observer(on_object_spawned);
    }
}

/// Observer that reacts to object spawning.
///
/// Layer 2 provides pre-computed vertices in `TiledObject` - Layer 3 uses
/// this data to create physics colliders without re-parsing.
fn on_object_spawned(
    trigger: On<ObjectSpawned>,
    object_query: Query<(&TiledObject, &Name)>,
) {
    let event = trigger.event();
    let Ok((tiled_object, object_name)) = object_query.get(event.entity) else {
        return;
    };

    info!(
        "🔧 SimplePhysicsPlugin: Object '{}' spawned",
        object_name
    );

    // Match on object shape - vertices already pre-computed by Layer 2!
    match tiled_object {
        TiledObject::Rectangle { width, height } => {
            info!(
                "  ✓ Added rectangle collider: {}x{}",
                width, height
            );
            // In real plugin: commands.entity(event.entity).insert(Collider::cuboid(*width, *height));
        }
        TiledObject::Polygon { vertices } => {
            info!(
                "  ✓ Added polygon collider with {} vertices",
                vertices.len()
            );
            // In real plugin: commands.entity(event.entity).insert(Collider::convex_hull(vertices.clone()));
        }
        TiledObject::Ellipse { width, height } => {
            info!(
                "  ✓ Added ellipse collider: {}x{}",
                width, height
            );
            // In real plugin: commands.entity(event.entity).insert(Collider::ball(width.max(*height) / 2.0));
        }
        TiledObject::Point => {
            info!("  ℹ Skipped point object (no collider needed)");
        }
        TiledObject::Polyline { vertices } => {
            info!(
                "  ✓ Added polyline collider with {} points",
                vertices.len()
            );
            // In real plugin: commands.entity(event.entity).insert(Collider::polyline(vertices.clone()));
        }
        TiledObject::Tile { .. } => {
            info!("  ℹ Skipped tile object (use sprite rendering)");
        }
        TiledObject::Text { .. } => {
            info!("  ℹ Skipped text object");
        }
    }
}

// ============================================================================
// Layer 3 Example: Custom Component Plugin
// ============================================================================

/// Plugin that reacts to `TiledClass` components added by Layer 2.
///
/// This demonstrates how Layer 3 plugins can respond to custom game
/// components that were auto-attached during spawning.
pub struct CustomComponentPlugin;

impl Plugin for CustomComponentPlugin {
    fn build(&self, app: &mut App) {
        // Two patterns for reacting to components:

        // 1. Observe general spawning events and check for components
        app.add_observer(on_object_spawned_check_components);

        // 2. Use Added<T> queries to react when components appear
        app.add_systems(Update, on_enemy_added);
    }
}

/// Observer pattern: Check for components when objects spawn.
fn on_object_spawned_check_components(
    trigger: On<ObjectSpawned>,
    query: Query<Option<&MergedProperties>>,
) {
    let event = trigger.event();
    if let Ok(Some(props)) = query.get(event.entity) {
        // Check if this object has specific properties
        if let Some(enemy_type) = props.get_string("enemy_type") {
            info!(
                "👾 CustomComponentPlugin: Enemy spawned with type '{}'",
                enemy_type
            );
        }

        if let Some(item_value) = props.get_i32("value") {
            info!(
                "💎 CustomComponentPlugin: Collectible spawned worth {} points",
                item_value
            );
        }
    }
}

/// Query pattern: React when specific components are added.
///
/// This example assumes an Enemy component exists (from `custom_components` example).
/// In a real plugin, you'd import your game components.
fn on_enemy_added(
    // NOTE: This would normally use Query<Entity, Added<Enemy>>
    // but we don't have Enemy in this example, so it's commented out
    _query: Query<Entity>,
) {
    // for entity in query.iter() {
    //     info!("👾 Enemy component added to entity {:?}", entity);
    //     // Add AI behavior, spawn health bar, etc.
    // }
}

// ============================================================================
// Key Takeaways
// ============================================================================

// 1. **Layer 2 provides structure, Layer 3 adds behavior**
//    - Layer 2: Entities, components, pre-processed data, events
//    - Layer 3: Sprites, colliders, AI, game logic
//
// 2. **Use observers for event-driven extension**
//    - TileLayerSpawned → Add rendering
//    - ObjectSpawned → Add physics
//    - ImageLayerSpawned → Add sprite
//
// 3. **Data is pre-processed for you**
//    - TileLayerData has resolved GIDs and flip flags
//    - TiledObject has computed vertices (no raw points)
//    - MergedProperties has template inheritance resolved
//
// 4. **Multiple Layer 3 plugins can coexist**
//    - Rendering plugin adds sprites
//    - Physics plugin adds colliders
//    - Game logic plugin adds AI components
//    - All observe the same Layer 2 events independently
//
// 5. **Layer 3 plugins are independent of each other**
//    - No coupling between rendering and physics
//    - Users can mix and match plugins
//    - Easy to swap implementations (e.g., bevy_ecs_tilemap vs native rendering)
