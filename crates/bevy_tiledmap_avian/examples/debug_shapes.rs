//! Debug shapes example - spawns test objects programmatically.
//!
//! This example demonstrates collider generation without requiring a Tiled map file.
//! It programmatically spawns various object shapes to visualize physics colliders.
//!
//! # What you'll see
//!
//! - Green outlines showing physics colliders (Avian debug gizmos)
//! - Rectangle, Ellipse, Polygon, and Point objects with colliders
//! - All objects are static with default physics properties

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_avian::prelude::*;
use bevy_tiledmap_core::components::object::TiledObject;
use bevy_tiledmap_core::events::ObjectSpawned;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        // Add Avian physics with debug visualization
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        // Add physics integration
        .add_plugins(TiledmapAvianPlugin::default())
        // Setup
        .add_systems(Startup, (setup_camera, spawn_test_objects))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 1000.0)));
}

fn spawn_test_objects(mut commands: Commands) {
    info!("🎮 Debug Shapes Example");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Spawning test objects with physics colliders...");

    // Rectangle object (centered at origin)
    let rect_entity = commands
        .spawn((
            TiledObject::Rectangle {
                width: 100.0,
                height: 50.0,
            },
            Transform::from_xyz(-150.0, 100.0, 0.0),
            Visibility::default(),
        ))
        .id();

    // Trigger event manually since we're not using the Layer 2 spawning system
    commands.trigger(ObjectSpawned {
        entity: rect_entity,
        map_entity: Entity::PLACEHOLDER,
        object_id: 1,
        properties: Default::default(),
    });

    info!("📦 Rectangle: 100x50 at (-150, 100)");

    // Ellipse object
    let ellipse_entity = commands
        .spawn((
            TiledObject::Ellipse {
                width: 80.0,
                height: 80.0,
            },
            Transform::from_xyz(150.0, 100.0, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.trigger(ObjectSpawned {
        entity: ellipse_entity,
        map_entity: Entity::PLACEHOLDER,
        object_id: 2,
        properties: Default::default(),
    });

    info!("⭕ Ellipse: 80x80 at (150, 100)");

    // Polygon object (triangle)
    let polygon_entity = commands
        .spawn((
            TiledObject::Polygon {
                vertices: vec![
                    Vec2::new(0.0, 40.0),
                    Vec2::new(-40.0, -20.0),
                    Vec2::new(40.0, -20.0),
                ],
            },
            Transform::from_xyz(-150.0, -100.0, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.trigger(ObjectSpawned {
        entity: polygon_entity,
        map_entity: Entity::PLACEHOLDER,
        object_id: 3,
        properties: Default::default(),
    });

    info!("🔺 Polygon: Triangle at (-150, -100)");

    // Polyline object (L-shape)
    let polyline_entity = commands
        .spawn((
            TiledObject::Polyline {
                vertices: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(60.0, 0.0),
                    Vec2::new(60.0, -40.0),
                ],
            },
            Transform::from_xyz(100.0, -100.0, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.trigger(ObjectSpawned {
        entity: polyline_entity,
        map_entity: Entity::PLACEHOLDER,
        object_id: 4,
        properties: Default::default(),
    });

    info!("📏 Polyline: L-shape at (100, -100)");

    // Point object
    let point_entity = commands
        .spawn((
            TiledObject::Point,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.trigger(ObjectSpawned {
        entity: point_entity,
        map_entity: Entity::PLACEHOLDER,
        object_id: 5,
        properties: Default::default(),
    });

    info!("📍 Point: Small circle at (0, 0)");

    info!("");
    info!("🟢 Green outlines = Avian physics colliders");
    info!("✨ All objects use default settings:");
    info!("   - Static rigid bodies");
    info!("   - Friction: 0.5");
    info!("   - Restitution: 0.0");
}
