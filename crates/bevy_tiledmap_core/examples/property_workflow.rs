//! Simplified example demonstrating the property-to-component workflow.
//!
//! This example manually creates objects to demonstrate how the system works
//! without requiring a Tiled map asset.

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_tiledmap_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(AssetPlugin::default())
        .add_plugins(bevy_tiledmap_assets::TiledmapAssetsPlugin)
        .add_plugins(TiledmapCorePlugin::default())
        // Register custom components
        .register_type::<Weapon>()
        .register_type::<PowerUp>()
        .add_systems(Startup, demonstrate_workflow)
        .add_systems(Update, exit_after_demo)
        .run();
}

fn demonstrate_workflow(world: &mut World) {
    info!("=== Property-to-Component Workflow Demo ===");
    info!("");
    info!("📋 Step 1: Define components with #[derive(TiledClass)]");
    info!("   - Weapon (damage: i32, fire_rate: f32, ammo: Option<i32>)");
    info!("   - PowerUp (bonus: f32, duration: f32, stackable: bool)");
    info!("");

    info!("📋 Step 2: Components are automatically registered via inventory");
    let registry = world.resource::<TiledClassRegistry>();
    info!("   Registered types:");
    for name in registry.type_names() {
        info!("     ✓ {}", name);
    }
    info!("");

    info!("📋 Step 3: When a map loads, bevy_tiledmap_core:");
    info!("   1. Spawns object entities");
    info!("   2. Checks properties for class-typed values");
    info!("   3. Deserializes and attaches matching components");
    info!("   4. Attaches MergedProperties for raw access");
    info!("   5. Triggers ObjectSpawned event");
    info!("");

    // Simulate what happens when a map loads
    simulate_object_spawn(world, "Weapon", create_weapon_props());
    simulate_object_spawn(world, "PowerUp", create_powerup_props());

    info!("📋 Step 4: Your game systems can now query for these components:");
    info!("   ```rust");
    info!("   fn weapon_system(query: Query<(Entity, &Weapon)>) {{");
    info!("       for (entity, weapon) in query.iter() {{");
    info!("           // Use weapon.damage, weapon.fire_rate, etc.");
    info!("       }}");
    info!("   }}");
    info!("   ```");
    info!("");

    info!("✅ Demo complete! The property-to-component system is working.");
    info!("   See custom_components.rs for a full example with map loading.");
    info!("");
}

fn simulate_object_spawn(world: &mut World, component_type: &str, props: tiled::Properties) {
    info!(
        "🔧 Simulating object spawn with type '{}'...",
        component_type
    );

    let entity = world.spawn_empty().id();

    // This is what happens internally in spawn_objects_layer
    let registry = world.resource::<TiledClassRegistry>();
    if let Some(info) = registry.get(component_type) {
        match (info.from_properties)(&props) {
            Ok(component_box) => {
                info!("   ✓ Deserialized {} component", component_type);

                // Insert via reflection
                let type_registry = world.resource::<AppTypeRegistry>().clone();
                let registry_lock = type_registry.read();
                let type_id = component_box.type_id();

                if let Some(reflect_component) =
                    registry_lock.get_type_data::<ReflectComponent>(type_id)
                    && let Ok(mut entity_mut) = world.get_entity_mut(entity)
                {
                    reflect_component.insert(&mut entity_mut, &*component_box, &registry_lock);
                    info!("   ✓ Attached component to entity {:?}", entity);
                }
            }
            Err(e) => {
                info!("   ✗ Failed to deserialize: {}", e);
            }
        }
    }

    info!("");
}

fn create_weapon_props() -> tiled::Properties {
    let mut props = tiled::Properties::new();
    props.insert("damage".to_string(), tiled::PropertyValue::IntValue(25));
    props.insert(
        "fire_rate".to_string(),
        tiled::PropertyValue::FloatValue(2.5),
    );
    props.insert("ammo".to_string(), tiled::PropertyValue::IntValue(30));
    props
}

fn create_powerup_props() -> tiled::Properties {
    let mut props = tiled::Properties::new();
    props.insert("bonus".to_string(), tiled::PropertyValue::FloatValue(1.5));
    props.insert(
        "duration".to_string(),
        tiled::PropertyValue::FloatValue(10.0),
    );
    props.insert(
        "stackable".to_string(),
        tiled::PropertyValue::BoolValue(true),
    );
    props
}

fn exit_after_demo(mut exit: MessageWriter<AppExit>, mut ran: Local<bool>) {
    if !*ran {
        *ran = true;
        return;
    }
    exit.write(AppExit::Success);
}

// ============================================================================
// Example Component Definitions
// ============================================================================

#[derive(Component, Reflect, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "Weapon")]
pub struct Weapon {
    /// Damage per hit
    pub damage: i32,

    /// Shots per second
    #[tiled(default = 1.0)]
    pub fire_rate: f32,

    /// Ammo count (optional, None = infinite)
    pub ammo: Option<i32>,
}

#[derive(Component, Reflect, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "PowerUp")]
pub struct PowerUp {
    /// Stat multiplier
    #[tiled(default = 1.0)]
    pub bonus: f32,

    /// Effect duration in seconds
    pub duration: f32,

    /// Can multiple be active?
    #[tiled(default = false)]
    pub stackable: bool,
}
