//! Layer spawning dispatcher.

use bevy::prelude::*;
use tiled::LayerType;

use crate::components::{LayerId, TiledLayer, TiledLayerMapOf};
use crate::events::{TileLayerSpawned, ObjectLayerSpawned, ImageLayerSpawned, GroupLayerSpawned};
use crate::spawn::{build_image_layer_data, build_tile_layer_data, spawn_objects_layer};
use crate::systems::SpawnContext;

/// Spawn a single layer entity with appropriate components.
///
/// Dispatches to type-specific spawning functions based on layer type.
/// Triggers appropriate layer spawned events for Layer 3 integration via observers.
///
/// # Arguments
///
/// * `commands` - Bevy commands for entity spawning
/// * `layer` - The layer from the map asset
/// * `map_entity` - Parent map entity (for relationship)
/// * `context` - Spawn context for asset access
/// * `type_registry` - App type registry for reflection-based component insertion
///
/// # Returns
///
/// The spawned layer entity
pub fn spawn_layer(
    commands: &mut Commands,
    layer: &tiled::Layer,
    map_entity: Entity,
    context: &SpawnContext,
    type_registry: &AppTypeRegistry,
) -> Entity {
    let layer_type = match layer.layer_type() {
        LayerType::Tiles(_) => TiledLayer::Tiles,
        LayerType::Objects(_) => TiledLayer::Objects,
        LayerType::Image(_) => TiledLayer::Image,
        LayerType::Group(_) => TiledLayer::Group,
    };

    // Calculate layer transform (offset, parallax will be added in Phase 3)
    let transform = Transform::from_xyz(
        layer.offset_x,
        -layer.offset_y,   // Invert Y
        layer.id() as f32, // Use layer ID as Z for ordering
    );

    // Spawn base layer entity and get ID immediately
    let layer_entity = commands
        .spawn((
            layer_type,
            LayerId(layer.id()),
            TiledLayerMapOf(map_entity),
            transform,
            Name::new(format!("Layer: {}", layer.name)),
        ))
        .id();

    // Add type-specific components/children and trigger events
    match layer.layer_type() {
        LayerType::Tiles(_) => {
            // Build tile data and attach to layer
            if let Some(tile_data) = build_tile_layer_data(layer, context) {
                commands.entity(layer_entity).insert(tile_data);
            }

            // Trigger TileLayerSpawned event
            commands.trigger(TileLayerSpawned {
                entity: layer_entity,
                map_entity,
                layer_id: layer.id(),
                properties: layer.properties.clone(),
            });
        }

        LayerType::Objects(_) => {
            // Spawn object entities as children
            let object_entities =
                spawn_objects_layer(commands, layer, map_entity, context, type_registry);
            if !object_entities.is_empty() {
                commands.entity(layer_entity).add_children(&object_entities);
            }

            // Trigger ObjectLayerSpawned event
            commands.trigger(ObjectLayerSpawned {
                entity: layer_entity,
                map_entity,
                layer_id: layer.id(),
                properties: layer.properties.clone(),
            });
        }

        LayerType::Image(_) => {
            // Build image data and attach to layer
            if let Some(image_data) = build_image_layer_data(layer, context) {
                commands.entity(layer_entity).insert(image_data);
            }

            // Trigger ImageLayerSpawned event
            commands.trigger(ImageLayerSpawned {
                entity: layer_entity,
                map_entity,
                layer_id: layer.id(),
                properties: layer.properties.clone(),
            });
        }

        LayerType::Group(group) => {
            // Recursively spawn child layers
            let mut child_layer_entities = Vec::new();
            for child_layer in group.layers() {
                let child_entity =
                    spawn_layer(commands, &child_layer, map_entity, context, type_registry);
                child_layer_entities.push(child_entity);
            }
            if !child_layer_entities.is_empty() {
                commands
                    .entity(layer_entity)
                    .add_children(&child_layer_entities);
            }

            // Trigger GroupLayerSpawned event
            commands.trigger(GroupLayerSpawned {
                entity: layer_entity,
                map_entity,
                layer_id: layer.id(),
                properties: layer.properties.clone(),
            });
        }
    }

    layer_entity
}
