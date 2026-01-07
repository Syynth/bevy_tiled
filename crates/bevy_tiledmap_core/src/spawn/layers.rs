//! Layer spawning dispatcher.

use bevy::prelude::*;
use tiled::LayerType;

use crate::components::{LayerId, TiledLayer, TiledLayerMapOf};
use crate::events::{TileLayerSpawned, ObjectLayerSpawned, ImageLayerSpawned, GroupLayerSpawned};
use crate::plugin::LayerZConfig;
use crate::spawn::{build_image_layer_data, build_tile_layer_data, spawn_objects_layer};
use crate::systems::SpawnContext;

/// Spawn a single layer entity with appropriate components.
///
/// Dispatches to type-specific spawning functions based on layer type.
/// Triggers appropriate layer spawned events for Layer 3 integration via observers.
///
/// # Z-Ordering
///
/// Content layers (tiles, objects, images) get sequential z values:
/// `z = config.offset + (counter * config.multiplier)`
///
/// Group layers get z=0 (they don't contribute to z-ordering, only their children do).
///
/// # Arguments
///
/// * `commands` - Bevy commands for entity spawning
/// * `layer` - The layer from the map asset
/// * `map_entity` - Parent map entity (for relationship)
/// * `context` - Spawn context for asset access
/// * `type_registry` - App type registry for reflection-based component insertion
/// * `z_counter` - Mutable counter for flat z-ordering across all content layers
/// * `z_config` - Configuration for z offset and multiplier
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
    z_counter: &mut usize,
    z_config: &LayerZConfig,
) -> Entity {
    let layer_type = match layer.layer_type() {
        LayerType::Tiles(_) => TiledLayer::Tiles,
        LayerType::Objects(_) => TiledLayer::Objects,
        LayerType::Image(_) => TiledLayer::Image,
        LayerType::Group(_) => TiledLayer::Group,
    };

    // Calculate Z value: groups get 0, content layers get sequential z values
    let z = if matches!(layer.layer_type(), LayerType::Group(_)) {
        0.0
    } else {
        let z = z_config.offset + (*z_counter as f32) * z_config.multiplier;
        *z_counter += 1;
        z
    };

    // Calculate layer transform (offset, parallax will be added in Phase 3)
    let transform = Transform::from_xyz(
        layer.offset_x,
        -layer.offset_y, // Invert Y for Tiled's Y-down to Bevy's Y-up
        z,
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
            // Build image data and attach to layer, only trigger event if data exists
            if let Some(image_data) = build_image_layer_data(layer, context) {
                commands.entity(layer_entity).insert(image_data);

                // Trigger ImageLayerSpawned event only when image data is present
                commands.trigger(ImageLayerSpawned {
                    entity: layer_entity,
                    map_entity,
                    layer_id: layer.id(),
                    properties: layer.properties.clone(),
                });
            }
        }

        LayerType::Group(group) => {
            // Recursively spawn child layers, skipping hidden ones
            // Children use is_top_level=false since their parent is already in positive Y space
            let mut child_layer_entities = Vec::new();
            for child_layer in group.layers() {
                if !child_layer.visible {
                    continue;
                }
                let child_entity =
                    spawn_layer(commands, &child_layer, map_entity, context, type_registry, z_counter, z_config);
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
