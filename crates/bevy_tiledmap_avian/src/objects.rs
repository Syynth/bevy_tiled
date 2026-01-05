//! Object collider generation.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::TiledTilesetAsset;
use bevy_tiledmap_core::components::object::TiledObject;
use bevy_tiledmap_core::events::ObjectSpawned;
use bevy_tiledmap_core::properties::registry::TiledClassRegistry;
use tiled::PropertyValue;

use crate::config::PhysicsConfig;
use crate::properties::PhysicsSettings;
use crate::shapes;

/// Observer that generates physics colliders for Tiled objects.
///
/// # Phase 2 Behavior (Property-Based Configuration)
///
/// Objects ONLY get colliders if they have a `physics_settings` property.
/// This provides opt-in control - you decide which objects should have physics.
///
/// The observer:
/// 1. Checks for `physics_settings` property (`avian::PhysicsSettings`)
/// 2. If not present, skips the object (no collider)
/// 3. If present, deserializes the `PhysicsSettings`
/// 4. Creates collider from object shape
/// 5. Attaches physics components based on `PhysicsSettings` values
///
/// # Example in Tiled
///
/// Add a custom class property to your object:
/// ```text
/// Property name: physics_settings
/// Property type: avian::PhysicsSettings
///
/// Values:
///   body_type: "Dynamic"
///   friction: 0.8
///   collision_groups: "player"
///   collision_mask: "ground,enemies"
/// ```
pub fn on_object_spawned(
    trigger: On<ObjectSpawned>,
    object_query: Query<&TiledObject>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    registry: Res<TiledClassRegistry>,
    type_registry: Res<AppTypeRegistry>,
    config: Res<PhysicsConfig>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok(object) = object_query.get(event.entity) else {
        return;
    };

    // Step 1: Resolve physics_settings and collider based on object type
    let (physics_settings, collider) = match object {
        TiledObject::Tile {
            tile_id,
            tileset_handle,
            width,
            height,
        } => {
            // For tile objects, merge properties from multiple sources:
            // 1. Tile properties (from tileset) - base for all objects in the tile
            // 2. Collision object properties (from tileset) - per-object override
            // 3. Tile object properties (from map) - instance-level override
            let Some(tileset) = tileset_assets.get(tileset_handle) else {
                return;
            };

            // Get collision shape and collision object properties
            let (collider, collision_props) =
                get_tile_collision_with_properties(tileset, *tile_id, *width, *height);

            // Merge properties: tile props (base) → collision props → object props (override)
            let merged_props = merge_tile_object_properties(
                tileset.tile_properties.get(tile_id), // base
                collision_props,                       // per-collision-object
                &event.properties,                     // instance override
            );

            // Resolve physics_settings from merged properties
            let Some(physics_settings) =
                resolve_physics_settings(&merged_props, &registry, &type_registry)
            else {
                return;
            };

            (physics_settings, collider)
        }
        _ => {
            // Non-tile objects: use object properties directly
            let Some(physics_settings) =
                resolve_physics_settings(&event.properties, &registry, &type_registry)
            else {
                return;
            };

            let Some(collider) = shapes::object_to_collider(object) else {
                warn!(
                    "Object {} has physics_settings but unsupported shape, skipping",
                    event.object_id
                );
                return;
            };

            (physics_settings, collider)
        }
    };

    // Step 3: Convert collision groups/mask to CollisionLayers via user callback
    let collision_layers = physics_settings.collision_layers(&config);

    // Step 4: Attach physics components based on PhysicsSettings
    let rigid_body = physics_settings.to_rigid_body();

    let mut entity_cmds = commands.entity(event.entity);
    entity_cmds.insert((
        rigid_body,
        collider,
        Friction::new(physics_settings.friction).with_combine_rule(CoefficientCombine::Average),
        Restitution::new(physics_settings.restitution)
            .with_combine_rule(CoefficientCombine::Average),
        collision_layers,
    ));

    // Add density for dynamic bodies
    if rigid_body == RigidBody::Dynamic {
        entity_cmds.insert(ColliderDensity(physics_settings.density));
    }

    // Add sensor component if configured
    if physics_settings.is_sensor {
        entity_cmds.insert(Sensor);
    }

    // Optional components
    if let Some(linear_damping) = physics_settings.linear_damping {
        entity_cmds.insert(LinearDamping(linear_damping));
    }
    if let Some(angular_damping) = physics_settings.angular_damping {
        entity_cmds.insert(AngularDamping(angular_damping));
    }
    if let Some(gravity_scale) = physics_settings.gravity_scale {
        entity_cmds.insert(GravityScale(gravity_scale));
    }
    if physics_settings.lock_rotation {
        entity_cmds.insert(LockedAxes::ROTATION_LOCKED);
    }

    info!(
        "Created collider for object {} with physics_settings (body_type: {:?}, friction: {}, restitution: {})",
        event.object_id,
        physics_settings.body_type,
        physics_settings.friction,
        physics_settings.restitution,
    );
}

/// Resolve `PhysicsSettings` from object properties.
///
/// Scans all properties for any with type `avian::PhysicsSettings`.
/// The property can have any name (e.g., "physics_settings", "collider", etc.)
///
/// This implements the opt-in filtering - only objects with this property get colliders.
fn resolve_physics_settings(
    properties: &tiled::Properties,
    registry: &TiledClassRegistry,
    type_registry: &AppTypeRegistry,
) -> Option<PhysicsSettings> {
    // Scan all properties for one with type avian::PhysicsSettings
    let class_props = properties.iter().find_map(|(_key, value)| {
        if let PropertyValue::ClassValue {
            property_type,
            properties: class_props,
        } = value
        {
            if property_type == "avian::PhysicsSettings" {
                return Some(class_props);
            }
        }
        None
    })?;

    // Get the TiledClassInfo for PhysicsSettings
    let class_info = registry.get("avian::PhysicsSettings")?;

    // Deserialize using the from_properties function
    // PhysicsSettings doesn't have Handle fields, so we pass None for AssetServer
    match (class_info.from_properties)(class_props, None) {
        Ok(boxed_reflect) => {
            // Downcast to PhysicsSettings using the type registry
            let registry_lock = type_registry.read();

            let type_id = boxed_reflect.get_represented_type_info()?.type_id();
            let registration = registry_lock.get(type_id)?;
            let reflect_from_reflect = registration.data::<ReflectFromReflect>()?;

            let settings: Box<dyn Reflect> = reflect_from_reflect.from_reflect(&*boxed_reflect)?;
            let settings = settings.downcast::<PhysicsSettings>().ok()?;

            Some(*settings)
        }
        Err(e) => {
            warn!("Failed to deserialize physics_settings: {}", e);
            None
        }
    }
}

/// Get tile collision shape and the collision object's properties.
///
/// Returns the collider and cloned properties from the first collision object in the tile.
/// If no collision shapes exist, returns a rectangle fallback and None.
fn get_tile_collision_with_properties(
    tileset: &TiledTilesetAsset,
    tile_id: u32,
    width: f32,
    height: f32,
) -> (Collider, Option<tiled::Properties>) {
    // Try to get tile collision data
    let Some(tile) = tileset.tileset.get_tile(tile_id) else {
        return (Collider::rectangle(width, height), None);
    };

    let Some(collision) = tile.collision.as_ref() else {
        return (Collider::rectangle(width, height), None);
    };

    let objects = collision.object_data();
    if objects.is_empty() {
        return (Collider::rectangle(width, height), None);
    }

    // Clone properties from first collision object (for single-shape tiles)
    // TODO: For compound shapes, consider merging properties from all objects
    let first_object_props = Some(objects[0].properties.clone());

    // Get the collider using existing shape logic
    let collider = shapes::get_tile_collision_shape(tileset, tile_id)
        .unwrap_or_else(|| Collider::rectangle(width, height));

    (collider, first_object_props)
}

/// Merge properties from multiple sources for tile objects.
///
/// Properties are merged with later sources overriding earlier:
/// 1. Tile properties (base for all objects in the tile)
/// 2. Collision object properties (per-object override)
/// 3. Tile object properties (instance-level override)
fn merge_tile_object_properties(
    tile_props: Option<&tiled::Properties>,
    collision_props: Option<tiled::Properties>,
    object_props: &tiled::Properties,
) -> tiled::Properties {
    let mut merged = tiled::Properties::default();

    // Apply tile properties first (base)
    if let Some(props) = tile_props {
        for (key, value) in props.iter() {
            merged.insert(key.clone(), value.clone());
        }
    }

    // Apply collision object properties (override)
    if let Some(props) = collision_props {
        for (key, value) in props.iter() {
            merged.insert(key.clone(), value.clone());
        }
    }

    // Apply tile object properties (highest priority)
    for (key, value) in object_props.iter() {
        merged.insert(key.clone(), value.clone());
    }

    merged
}
