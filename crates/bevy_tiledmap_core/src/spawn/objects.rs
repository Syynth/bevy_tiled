//! Object layer spawning.

use bevy::prelude::*;
use tiled::{LayerType, ObjectShape, PropertyValue};

use crate::components::TiledObjectMapOf;
use crate::components::object::{ObjectId, TiledObject};
use crate::events::ObjectSpawned;
use crate::properties::MergedProperties;
use crate::systems::SpawnContext;

/// Spawn object entities for an object layer.
///
/// Pre-computes shape vertices, resolves tile references, sets up transforms.
/// Automatically attaches `MergedProperties` and any registered `TiledClass` components.
/// Triggers `ObjectSpawned` events for Layer 3 integration via observers.
///
/// # Arguments
///
/// * `commands` - Bevy commands for entity spawning
/// * `layer` - The object layer from the map asset
/// * `map_entity` - Parent map entity (for relationship)
/// * `context` - Spawn context for tileset lookups and property access
/// * `type_registry` - App type registry for reflection-based component insertion
///
/// # Returns
///
/// Vec of spawned object entities
pub fn spawn_objects_layer(
    commands: &mut Commands,
    layer: &tiled::Layer,
    map_entity: Entity,
    context: &SpawnContext,
    type_registry: &AppTypeRegistry,
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

        // Get merged properties (template + object)
        let merged_props = context.get_merged_object_properties(&object);

        // Spawn object entity with base components
        let mut entity_cmd = commands.spawn((
            tiled_object,
            ObjectId(object.id()),
            TiledObjectMapOf(map_entity),
            transform,
            Name::new(format!("Object: {}", object.name)),
        ));

        // Attach MergedProperties for raw property access
        entity_cmd.insert(MergedProperties::new(merged_props.clone()));

        // Auto-attach registered TiledClass components
        attach_registered_components(&mut entity_cmd, merged_props, context, type_registry);

        let entity_id = entity_cmd.id();
        object_entities.push(entity_id);

        // Trigger ObjectSpawned event for Layer 3 plugins (via observers)
        commands.trigger(ObjectSpawned {
            entity: entity_id,
            map_entity,
            object_id: object.id(),
            properties: merged_props.clone(),
        });
    }

    object_entities
}

/// Attach registered components from class-typed properties.
///
/// Iterates through the object's properties looking for class-typed values.
/// For each class property, attempts to deserialize and attach the corresponding component.
fn attach_registered_components(
    entity_cmd: &mut EntityCommands,
    properties: &tiled::Properties,
    context: &SpawnContext,
    type_registry: &AppTypeRegistry,
) {
    // Collect components to insert (can't insert during iteration due to borrow checker)
    let mut components_to_insert: Vec<Box<dyn Reflect>> = Vec::new();

    // Iterate all properties looking for class-typed ones
    for (key, value) in properties.iter() {
        // Check if this is a class-typed property
        if let PropertyValue::ClassValue {
            property_type,
            properties: class_props,
        } = value
        {
            // Try to find this class in the registry
            if let Some(info) = context.registry.get(property_type) {
                // Call the generated deserialization function
                match (info.from_properties)(class_props) {
                    Ok(component_box) => {
                        // Verify it has ReflectComponent
                        let type_id = component_box.type_id();
                        let registry_lock = type_registry.read();

                        if registry_lock
                            .get_type_data::<ReflectComponent>(type_id)
                            .is_some()
                        {
                            components_to_insert.push(component_box);
                            debug!(
                                "Queued component '{}' for attachment (property: '{}')",
                                property_type, key
                            );
                        } else {
                            warn!(
                                "Type '{}' is registered but missing ReflectComponent. \
                                Did you forget #[reflect(Component)]?",
                                property_type
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize component '{}' for property '{}': {}",
                            property_type, key, e
                        );
                    }
                }
            } else {
                debug!(
                    "Class property '{}' has type '{}' which is not registered. \
                    Add #[derive(TiledClass)] to register it.",
                    key, property_type
                );
            }
        }
    }

    // Insert all collected components via custom command
    if !components_to_insert.is_empty() {
        let entity = entity_cmd.id();
        let type_registry_clone = type_registry.clone();

        entity_cmd.commands().queue(move |world: &mut World| {
            let registry = type_registry_clone.read();
            for component_box in components_to_insert {
                let type_id = component_box.type_id();
                if let Some(reflect_component) = registry.get_type_data::<ReflectComponent>(type_id)
                    && let Ok(mut entity_mut) = world.get_entity_mut(entity)
                {
                    reflect_component.insert(&mut entity_mut, &*component_box, &registry);
                }
            }
        });
    }
}
