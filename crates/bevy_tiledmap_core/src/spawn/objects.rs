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
        // Check if this is a tile object first
        let tiled_object = if let Some(tile_data) = object.tile_data() {
            // This is a tile object - get tile info
            let tile_id = tile_data.id();

            // Get dimensions from the object shape (tile objects have Rect shape)
            let (obj_width, obj_height) = match &object.shape {
                ObjectShape::Rect { width, height } => (*width, *height),
                _ => {
                    warn!("Tile object has non-Rect shape, using 0 dimensions");
                    (0.0, 0.0)
                }
            };

            // Find the tileset by checking which one contains this tile
            let tileset_result = find_tileset_for_tile_object(context, &tile_data);

            match tileset_result {
                Some((tileset_handle, _first_gid)) => TiledObject::Tile {
                    tile_id,
                    tileset_handle,
                    width: obj_width,
                    height: obj_height,
                },
                None => {
                    warn!(
                        "Could not find tileset for tile object '{}' (tile_id: {})",
                        object.name, tile_id
                    );
                    // Fall through to shape-based handling
                    convert_object_shape(&object.shape)
                }
            }
        } else {
            // Regular shape-based object
            convert_object_shape(&object.shape)
        };

        // Calculate transform
        // Tiled uses corner origin with Y increasing downward
        // Bevy uses center origin with Y increasing upward (positive Y space)
        //
        // We convert from Tiled coordinates to Bevy's positive Y coordinate system:
        // - Map origin (0,0) is at bottom-left in Bevy world space
        // - Y increases upward
        // - For regular objects: Tiled anchor is TOP-left
        // - For tile objects: Tiled anchor is BOTTOM-left
        let (obj_width, obj_height) = match &object.shape {
            ObjectShape::Rect { width, height } => (*width, *height),
            _ => (0.0, 0.0),
        };

        // Map dimensions for Y-flip calculation
        let map_pixel_height =
            context.map_asset.map.height as f32 * context.map_asset.map.tile_height as f32;

        // Calculate center position in Bevy coordinates (positive Y space)
        let (center_x, center_y) = if object.tile_data().is_some() {
            // Tile objects: anchor is at BOTTOM-left, tile extends UP
            // Center X = x + width/2
            // Tiled Y is from top, Bevy Y is from bottom
            // Object center in Tiled coords = y - height/2 (since tile extends up)
            // Bevy Y = map_height - tiled_y_center
            (
                object.x + obj_width / 2.0,
                map_pixel_height - (object.y - obj_height / 2.0),
            )
        } else {
            // Regular objects: anchor is at TOP-left, object extends DOWN
            // Center X = x + width/2
            // Object center in Tiled coords = y + height/2
            // Bevy Y = map_height - tiled_y_center
            (
                object.x + obj_width / 2.0,
                map_pixel_height - (object.y + obj_height / 2.0),
            )
        };

        let transform = Transform::from_xyz(center_x, center_y, 0.0)
            // Tiled rotation is clockwise in degrees, Bevy is counter-clockwise in radians
            .with_rotation(Quat::from_rotation_z(-object.rotation.to_radians()));

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

/// Convert an ObjectShape to TiledObject.
///
/// Transforms vertices from Tiled's coordinate system (Y-down) to Bevy's (Y-up).
/// Vertices are relative to the object's transform position.
fn convert_object_shape(shape: &ObjectShape) -> TiledObject {
    match shape {
        ObjectShape::Rect { width, height } => TiledObject::Rectangle {
            width: *width,
            height: *height,
        },

        ObjectShape::Ellipse { width, height } => TiledObject::Ellipse {
            width: *width,
            height: *height,
        },

        ObjectShape::Polyline { points } => {
            // Flip Y for Bevy's Y-up coordinate system
            let vertices: Vec<Vec2> = points.iter().map(|(x, y)| Vec2::new(*x, -*y)).collect();
            TiledObject::Polyline { vertices }
        }

        ObjectShape::Polygon { points } => {
            // Flip Y for Bevy's Y-up coordinate system
            let vertices: Vec<Vec2> = points.iter().map(|(x, y)| Vec2::new(*x, -*y)).collect();
            TiledObject::Polygon { vertices }
        }

        ObjectShape::Point(_, _) => TiledObject::Point,

        ObjectShape::Text { .. } => TiledObject::Text {},
    }
}

/// Find the tileset for a tile object.
///
/// Tile objects reference tiles via TilesetLocation which can be:
/// - Map(index) - index into the map's tileset list (matches our HashMap key)
/// - Template(Arc<Tileset>) - tileset from a template
fn find_tileset_for_tile_object(
    context: &SpawnContext,
    tile_data: &tiled::ObjectTileData,
) -> Option<(Handle<bevy_tiledmap_assets::prelude::TiledTilesetAsset>, u32)> {
    use tiled::TilesetLocation;

    match tile_data.tileset_location() {
        TilesetLocation::Map(tileset_index) => {
            // Direct lookup by tileset index (matches our HashMap key)
            context
                .get_tileset_by_index(*tileset_index as u32)
                .map(|tileset_ref| (tileset_ref.handle.clone(), tileset_ref.first_gid))
        }
        TilesetLocation::Template(tileset_arc) => {
            // Template tileset - find by matching the tileset source path
            for (_tileset_index, tileset_ref) in context.map_asset.tilesets.iter() {
                if let Some(tileset_asset) = context.tileset_assets.get(&tileset_ref.handle) {
                    if tileset_asset.tileset.source == tileset_arc.source {
                        return Some((tileset_ref.handle.clone(), tileset_ref.first_gid));
                    }
                }
            }
            None
        }
    }
}

