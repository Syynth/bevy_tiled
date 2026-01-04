//! Object layer spawning.

use bevy::prelude::*;
use tiled::{LayerType, ObjectShape};

use crate::components::TiledObjectMapOf;
use crate::components::object::{ObjectId, TiledObject};
use crate::systems::SpawnContext;

/// Spawn object entities for an object layer.
///
/// Pre-computes shape vertices, resolves tile references, sets up transforms.
///
/// # Arguments
///
/// * `commands` - Bevy commands for entity spawning
/// * `layer` - The object layer from the map asset
/// * `map_entity` - Parent map entity (for relationship)
/// * `context` - Spawn context for tileset lookups
///
/// # Returns
///
/// Vec of spawned object entities
pub fn spawn_objects_layer(
    commands: &mut Commands,
    layer: &tiled::Layer,
    map_entity: Entity,
    _context: &SpawnContext,
) -> Vec<Entity> {
    // Only process object layers
    let LayerType::Objects(object_layer) = layer.layer_type() else {
        return Vec::new();
    };

    let mut object_entities = Vec::new();

    for object in object_layer.objects() {
        // Convert object shape to TiledObject enum with pre-computed data
        let tiled_object = match &object.shape {
            ObjectShape::Rect { width, height } => TiledObject::Rectangle {
                width: *width,
                height: *height,
            },

            ObjectShape::Ellipse { width, height } => TiledObject::Ellipse {
                width: *width,
                height: *height,
            },

            ObjectShape::Polyline { points } => {
                // Pre-compute vertices (convert f64 → Vec2)
                let vertices: Vec<Vec2> = points.iter().map(|(x, y)| Vec2::new(*x, *y)).collect();

                TiledObject::Polyline { vertices }
            }

            ObjectShape::Polygon { points } => {
                // Pre-compute vertices (convert f64 → Vec2)
                let vertices: Vec<Vec2> = points.iter().map(|(x, y)| Vec2::new(*x, *y)).collect();

                TiledObject::Polygon { vertices }
            }

            ObjectShape::Point(_, _) => TiledObject::Point,

            ObjectShape::Text { .. } => TiledObject::Text {},
        };

        // Calculate transform (Tiled Y-axis is inverted)
        let transform = Transform::from_xyz(
            object.x, -object.y, // Invert Y
            0.0,
        )
        .with_rotation(Quat::from_rotation_z(object.rotation.to_radians()));

        // Spawn object entity
        let object_entity = commands
            .spawn((
                tiled_object,
                ObjectId(object.id()),
                TiledObjectMapOf(map_entity),
                transform,
                Name::new(format!("Object: {}", object.name)),
            ))
            .id();

        object_entities.push(object_entity);
    }

    object_entities
}
