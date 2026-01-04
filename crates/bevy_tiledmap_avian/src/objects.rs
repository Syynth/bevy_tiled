//! Object collider generation.

use avian2d::prelude::*;
use bevy::prelude::*;
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
    registry: Res<TiledClassRegistry>,
    type_registry: Res<AppTypeRegistry>,
    config: Res<PhysicsConfig>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok(object) = object_query.get(event.entity) else {
        return;
    };

    // Step 1: Check for physics_settings property (opt-in filtering)
    // Objects ONLY get colliders if they have this property
    let Some(physics_settings) =
        resolve_physics_settings(&event.properties, &registry, &type_registry)
    else {
        // No physics_settings property = no collider
        return;
    };

    // Step 2: Create collider from object shape
    let Some(collider) = shapes::object_to_collider(object) else {
        // Text objects and other unsupported shapes are skipped
        warn!(
            "Object {} has physics_settings but unsupported shape, skipping",
            event.object_id
        );
        return;
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
/// Returns `Some(PhysicsSettings)` if the object has a `physics_settings` property
/// with type `avian::PhysicsSettings`, otherwise `None`.
///
/// This implements the opt-in filtering - only objects with this property get colliders.
fn resolve_physics_settings(
    properties: &tiled::Properties,
    registry: &TiledClassRegistry,
    type_registry: &AppTypeRegistry,
) -> Option<PhysicsSettings> {
    // Look for physics_settings property
    let property_value = properties.get("physics_settings")?;

    // It must be a ClassValue
    let PropertyValue::ClassValue {
        property_type,
        properties: class_props,
    } = property_value
    else {
        warn!("physics_settings property is not a ClassValue, ignoring");
        return None;
    };

    // Verify it's the correct type
    if property_type != "avian::PhysicsSettings" {
        warn!(
            "physics_settings has wrong type '{}', expected 'avian::PhysicsSettings'",
            property_type
        );
        return None;
    }

    // Get the TiledClassInfo for PhysicsSettings
    let class_info = registry.get("avian::PhysicsSettings")?;

    // Deserialize using the from_properties function
    match (class_info.from_properties)(class_props) {
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
