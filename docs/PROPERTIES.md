# Property-to-Component System

The property-to-component system automatically converts Tiled custom properties into Bevy components at runtime. This allows you to define gameplay components directly in the Tiled editor and have them automatically attached to spawned entities.

## Table of Contents

- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Defining Components](#defining-components)
- [Supported Types](#supported-types)
- [Attributes](#attributes)
- [Template Inheritance](#template-inheritance)
- [Accessing Properties](#accessing-properties)
- [Tiled Editor Integration](#tiled-editor-integration)
- [Layer 3 Integration](#layer-3-integration)
- [Examples](#examples)

## Quick Start

**1. Define a component with `#[derive(TiledClass)]`:**

```rust
use bevy::prelude::*;
use bevy_tiled_core::prelude::*;

#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Player")]
pub struct Player {
    /// Player health points
    #[tiled(default = 100.0)]
    pub health: f32,

    /// Movement speed
    #[tiled(default = 5.0)]
    pub speed: f32,

    /// Optional team ID
    pub team: Option<i32>,
}
```

**2. Register the component:**

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyTiledAssetsPlugin)
        .add_plugins(BevyTiledCorePlugin::default())
        .run();
}
```

**3. Use in Tiled:**

1. Export types: Configure the plugin to export JSON
2. Import in Tiled: View → Custom Types → Import
3. Add properties: Objects → Add Property → Select "game::Player"
4. Set values: The component fields appear as editable properties

**4. The component is automatically attached when the map loads!**

```rust
fn player_system(query: Query<(Entity, &Player, &Transform)>) {
    for (entity, player, transform) in query.iter() {
        info!("Player entity {:?} at {:?}", entity, transform.translation);
        info!("  Health: {}, Speed: {}", player.health, player.speed);
    }
}
```

## How It Works

The property-to-component system operates in several phases:

### Phase 1: Compile-Time Registration

When you add `#[derive(TiledClass)]` to a component:

1. The macro generates a deserialization function
2. Field metadata is collected (name, type, default value)
3. The type is registered via the `inventory` crate
4. All registered types are available at startup

### Phase 2: Map Loading

When a map with objects is loaded:

1. **Spawn entities**: Object entities are created with base components (`TiledObject`, `Transform`, etc.)
2. **Check properties**: Each object's properties are examined for class-typed values
3. **Deserialize components**: Matching components are deserialized from properties
4. **Attach via reflection**: Components are dynamically attached using Bevy's reflection system
5. **Add MergedProperties**: Raw property data is stored for Layer 3 access
6. **Trigger events**: `ObjectSpawned` events fire for observer integration

### Phase 3: Runtime Access

Your game systems can now:

- Query for auto-attached components
- React to spawn events via observers
- Access raw properties via `MergedProperties`

## Defining Components

### Basic Component

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Enemy")]
pub struct Enemy {
    pub damage: i32,
    pub speed: f32,
}
```

**Requirements:**
- `#[derive(Component)]` - Must be a Bevy component
- `#[derive(Reflect)]` - Required for dynamic attachment
- `#[derive(TiledClass)]` - Enables property-to-component system
- `#[reflect(Component)]` - Registers reflection data
- `#[tiled(name = "...")]` - Name that appears in Tiled editor

### With Default Values

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Door")]
pub struct Door {
    /// Whether the door starts locked
    #[tiled(default = false)]
    pub locked: bool,

    /// Door health points
    #[tiled(default = 100.0)]
    pub health: f32,

    /// Required key ID (no default - must be set in Tiled)
    pub key_id: u32,
}
```

### With Optional Fields

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Collectible")]
pub struct Collectible {
    /// Points awarded (required)
    pub points: i32,

    /// Optional category tag
    pub category: Option<String>,

    /// Optional respawn delay
    pub respawn_time: Option<f32>,
}
```

Optional fields (`Option<T>`) automatically default to `None` if not present in Tiled.

### Skipping Fields

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::NPC")]
pub struct NPC {
    /// Synced from Tiled
    pub dialogue_id: String,

    /// Runtime-only state (not in Tiled)
    #[tiled(skip)]
    pub dialogue_state: DialogueState,
}
```

Fields marked with `#[tiled(skip)]` are initialized with `Default::default()` and don't appear in Tiled.

## Supported Types

### Primitive Types

| Rust Type | Tiled Type | Example |
|-----------|------------|---------|
| `bool` | `bool` | `true`, `false` |
| `i32`, `i64`, `i16`, `i8` | `int` | `42`, `-100` |
| `u32`, `u64`, `u16`, `u8` | `int` | `100` (cast to signed) |
| `f32`, `f64` | `float` | `3.14`, `-0.5` |
| `String` | `string` | `"Hello"` |

### Bevy Types

| Rust Type | Tiled Type | Format |
|-----------|------------|--------|
| `Color` | `color` | RGBA color picker |
| `Vec2` | `string` | `"x,y"` (manual entry) |
| `Vec3` | `string` | `"x,y,z"` (manual entry) |

**Note:** Tiled doesn't have native vector types. Vec2/Vec3 are stored as comma-separated strings and must implement custom `FromTiledProperty`.

### Optional Types

Any type `T` can be wrapped in `Option<T>`:

```rust
pub struct Component {
    pub optional_damage: Option<i32>,      // None if not set
    pub optional_name: Option<String>,     // None if not set
    pub optional_color: Option<Color>,     // None if not set
}
```

### Custom Enums

For custom enums, implement `FromTiledProperty`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum EnemyType {
    Grunt,
    Elite,
    Boss,
}

impl FromTiledProperty for EnemyType {
    fn from_property(value: &tiled::PropertyValue) -> Option<Self> {
        match value {
            tiled::PropertyValue::StringValue(s) => match s.as_str() {
                "Grunt" => Some(EnemyType::Grunt),
                "Elite" => Some(EnemyType::Elite),
                "Boss" => Some(EnemyType::Boss),
                _ => None,
            },
            _ => None,
        }
    }
}
```

Then use it in your component:

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Enemy")]
pub struct Enemy {
    pub enemy_type: EnemyType,  // Appears as string dropdown in Tiled
}
```

## Attributes

### Struct Attributes

#### `#[tiled(name = "...")]`

**Required.** Sets the class name that appears in Tiled editor.

```rust
#[tiled(name = "game::Player")]  // Shows as "game::Player" in Tiled
```

**Naming Convention:** Use `namespace::TypeName` format (e.g., `game::Player`, `physics::Collider`) to organize types.

### Field Attributes

#### `#[tiled(default = ...)]`

Sets a default value if the property is missing in Tiled.

```rust
pub struct Component {
    #[tiled(default = 100.0)]
    pub health: f32,              // Defaults to 100.0

    #[tiled(default = true)]
    pub enabled: bool,            // Defaults to true

    #[tiled(default = 0)]
    pub team_id: i32,             // Defaults to 0
}
```

#### `#[tiled(skip)]`

Excludes the field from Tiled serialization. Uses `Default::default()` at runtime.

```rust
pub struct Component {
    pub tiled_value: i32,     // In Tiled

    #[tiled(skip)]
    pub runtime_state: State,  // Runtime-only, not in Tiled
}
```

## Template Inheritance

Tiled templates allow property reuse. The `tiled` crate (v0.15+) automatically merges template properties during parsing.

**Template (EnemyTemplate.tx):**
```json
{
  "type": "template",
  "object": {
    "type": "game::Enemy",
    "properties": [
      { "name": "damage", "type": "int", "value": 10 },
      { "name": "speed", "type": "float", "value": 2.0 }
    ]
  }
}
```

**Object in Map (overrides damage):**
```json
{
  "template": "EnemyTemplate.tx",
  "properties": [
    { "name": "damage", "type": "int", "value": 25 }
  ]
}
```

**Result:** Object has `damage: 25` (overridden) and `speed: 2.0` (from template).

The property-to-component system handles this automatically—no manual merging needed!

## Accessing Properties

### Via Auto-Attached Components

The primary way to access data:

```rust
fn gameplay_system(query: Query<(Entity, &Player, &Transform)>) {
    for (entity, player, transform) in query.iter() {
        // Use component fields directly
        if player.health <= 0.0 {
            // Handle player death
        }
    }
}
```

### Via MergedProperties

For raw property access (useful for Layer 3 plugins):

```rust
use bevy_tiled_core::prelude::*;

fn inspect_properties(query: Query<(Entity, &MergedProperties, &TiledObject)>) {
    for (entity, props, object) in query.iter() {
        // Type-safe accessors
        if let Some(enabled) = props.get_bool("enabled") {
            info!("Object is {}", if enabled { "enabled" } else { "disabled" });
        }

        if let Some(priority) = props.get_i32("priority") {
            info!("Priority: {}", priority);
        }

        if let Some(name) = props.get_string("name") {
            info!("Name: {}", name);
        }

        // Iterate all properties
        for (key, value) in props.iter() {
            info!("  {}: {:?}", key, value);
        }
    }
}
```

**MergedProperties API:**
- `get_bool(key) -> Option<bool>`
- `get_i32(key) -> Option<i32>`
- `get_f32(key) -> Option<f32>`
- `get_string(key) -> Option<&str>`
- `get_color(key) -> Option<tiled::Color>`
- `iter() -> Iterator<Item = (&String, &PropertyValue)>`

## Tiled Editor Integration

### Exporting Type Definitions

**1. Configure the plugin:**

```rust
use bevy_tiled_core::{BevyTiledCorePlugin, BevyTiledCoreConfig};

App::new()
    .add_plugins(BevyTiledCorePlugin::new(BevyTiledCoreConfig {
        export_types_path: Some("assets/tiled-types.json".into()),
    }))
    .run();
```

**2. Run your game once** - The JSON file is generated at startup

**3. Import in Tiled:**
- Open Tiled
- View → Custom Types → Import Custom Types
- Select `assets/tiled-types.json`
- Your types now appear in property dropdowns!

### Using Custom Types in Tiled

**Adding a custom property:**

1. Select an object in Tiled
2. In the Properties panel, click **"+"** → **Add Property**
3. Choose **"Custom Type"** from the dropdown
4. Select your type (e.g., `game::Player`)
5. The component's fields appear as editable properties

**Editing fields:**

- **Bool fields:** Checkbox
- **Int/Float fields:** Number input
- **String fields:** Text input
- **Color fields:** Color picker
- **Enums:** String input (type the variant name)

**Example in Tiled:**

```
Object Properties:
  + game::Player
    ├─ health: 100.0  [float]
    ├─ speed: 5.0     [float]
    └─ team: 1        [int]
```

### JSON Format

The exported JSON follows Tiled's custom types specification:

```json
{
  "version": "1.10",
  "propertyTypes": [
    {
      "id": 1,
      "name": "game::Player",
      "type": "class",
      "members": [
        {
          "name": "health",
          "type": "float",
          "value": 100.0
        },
        {
          "name": "speed",
          "type": "float",
          "value": 5.0
        }
      ]
    }
  ]
}
```

## Layer 3 Integration

Layer 3 plugins (rendering, physics, etc.) can hook into the spawning process using observers and events.

### Observing ObjectSpawned Events

```rust
use bevy_tiled_core::prelude::*;

fn setup(mut commands: Commands) {
    // React to object spawns
    commands.add_observer(on_object_spawned);
}

fn on_object_spawned(
    trigger: On<ObjectSpawned>,
    objects: Query<&TiledObject>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Check if object should have physics
    if let Some(tiled::PropertyValue::BoolValue(true)) =
        event.properties.get("has_physics")
    {
        commands.entity(event.entity).insert(RigidBody::Dynamic);
    }

    // Access object shape for collider generation
    if let Ok(object) = objects.get(event.entity) {
        match object {
            TiledObject::Rectangle { width, height } => {
                // Add box collider
            }
            TiledObject::Polygon { vertices } => {
                // Add polygon collider
            }
            _ => {}
        }
    }
}
```

### Available Events

All events are triggered via Bevy's observer system:

- **`ObjectSpawned`** - Fired for each spawned object
  - `entity: Entity` - The spawned object
  - `map_entity: Entity` - Parent map
  - `object_id: u32` - Tiled object ID
  - `properties: Properties` - Merged properties

- **`TileLayerSpawned`** - Fired when tile layers spawn
- **`ObjectLayerSpawned`** - Fired when object layers spawn
- **`ImageLayerSpawned`** - Fired when image layers spawn
- **`GroupLayerSpawned`** - Fired when group layers spawn

All layer events include:
- `entity: Entity` - The layer entity
- `map_entity: Entity` - Parent map
- `layer_id: u32` - Tiled layer ID
- `properties: Properties` - Layer properties

### Conditional Component Attachment

```rust
fn on_object_spawned(
    trigger: On<ObjectSpawned>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Add rendering components based on properties
    if let Some(tiled::PropertyValue::BoolValue(true)) =
        event.properties.get("glow")
    {
        commands.entity(event.entity).insert(GlowEffect::default());
    }

    // Add audio based on properties
    if let Some(tiled::PropertyValue::StringValue(sound)) =
        event.properties.get("ambient_sound")
    {
        commands.entity(event.entity).insert(AmbientSound {
            path: sound.clone(),
        });
    }
}
```

### Accessing MergedProperties

Every object and layer gets a `MergedProperties` component:

```rust
fn property_based_logic(
    query: Query<(Entity, &MergedProperties), Added<MergedProperties>>,
) {
    for (entity, props) in query.iter() {
        // Data-driven gameplay logic
        if let Some(spawn_delay) = props.get_f32("spawn_delay") {
            // Delay spawning
        }

        if let Some(faction) = props.get_string("faction") {
            // Assign to faction
        }
    }
}
```

## Examples

### Complete Game Component

```rust
use bevy::prelude::*;
use bevy_tiled_core::prelude::*;

/// Player component with various property types
#[derive(Component, Reflect, TiledClass, Debug)]
#[reflect(Component)]
#[tiled(name = "game::Player")]
pub struct Player {
    /// Starting health points
    #[tiled(default = 100.0)]
    pub health: f32,

    /// Movement speed in units/second
    #[tiled(default = 5.0)]
    pub speed: f32,

    /// Player name (optional)
    pub name: Option<String>,

    /// Team/faction ID
    #[tiled(default = 0)]
    pub team_id: i32,

    /// Player color tint
    #[tiled(default = Color::srgb(1.0, 1.0, 1.0))]
    pub color: Color,

    /// Runtime animation state (not in Tiled)
    #[tiled(skip)]
    pub animation_state: AnimationState,
}

#[derive(Default, Reflect)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    Running,
}
```

### Physics Integration Example

```rust
use bevy::prelude::*;
use bevy_tiled_core::prelude::*;

/// Physics properties component
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "physics::Body")]
pub struct PhysicsBody {
    /// Body type
    pub body_type: BodyType,

    /// Mass in kilograms
    #[tiled(default = 1.0)]
    pub mass: f32,

    /// Friction coefficient
    #[tiled(default = 0.5)]
    pub friction: f32,

    /// Restitution (bounciness)
    #[tiled(default = 0.0)]
    pub restitution: f32,
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum BodyType {
    Static,
    Dynamic,
    Kinematic,
}

impl FromTiledProperty for BodyType {
    fn from_property(value: &tiled::PropertyValue) -> Option<Self> {
        match value {
            tiled::PropertyValue::StringValue(s) => match s.as_str() {
                "Static" => Some(BodyType::Static),
                "Dynamic" => Some(BodyType::Dynamic),
                "Kinematic" => Some(BodyType::Kinematic),
                _ => None,
            },
            _ => None,
        }
    }
}

// Observer that adds physics engine components
fn setup_physics_observer(mut commands: Commands) {
    commands.add_observer(attach_physics_components);
}

fn attach_physics_components(
    trigger: On<ObjectSpawned>,
    query: Query<(&PhysicsBody, &TiledObject)>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;

    if let Ok((body, object)) = query.get(entity) {
        // Attach physics engine components based on TiledClass data
        match body.body_type {
            BodyType::Static => {
                commands.entity(entity).insert(RigidBody::Static);
            }
            BodyType::Dynamic => {
                commands.entity(entity).insert((
                    RigidBody::Dynamic,
                    Mass(body.mass),
                    Friction(body.friction),
                    Restitution(body.restitution),
                ));
            }
            BodyType::Kinematic => {
                commands.entity(entity).insert(RigidBody::Kinematic);
            }
        }

        // Generate collider from object shape
        match object {
            TiledObject::Rectangle { width, height } => {
                commands.entity(entity).insert(
                    Collider::cuboid(*width / 2.0, *height / 2.0)
                );
            }
            TiledObject::Polygon { vertices } => {
                commands.entity(entity).insert(
                    Collider::polygon(vertices.clone())
                );
            }
            _ => {}
        }
    }
}
```

### Querying Components

```rust
fn player_movement_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    for (player, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * player.speed * time.delta_secs();
        }
    }
}

fn enemy_ai_system(
    query: Query<(&Enemy, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    for (enemy, transform) in query.iter() {
        // Use enemy component data
        if let Ok(player_transform) = player_query.get_single() {
            let distance = transform.translation.distance(player_transform.translation);

            if distance < enemy.detection_range {
                // Player detected, chase at enemy.patrol_speed
            }
        }
    }
}
```

## Best Practices

### Naming Conventions

Use namespaced names to organize types:

```rust
#[tiled(name = "game::Player")]      // Gameplay components
#[tiled(name = "physics::Body")]     // Physics components
#[tiled(name = "ai::Patrol")]        // AI components
#[tiled(name = "fx::ParticleEmitter")] // Visual effects
```

### Default Values

Provide sensible defaults for common cases:

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Health")]
pub struct Health {
    #[tiled(default = 100.0)]  // Good: Reasonable default
    pub max_health: f32,

    #[tiled(default = 100.0)]  // Good: Start at full health
    pub current_health: f32,

    pub regeneration_rate: f32,  // No default: Must be set intentionally
}
```

### Documentation

Document fields in Rust—they help both code readers and Tiled users:

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Weapon")]
pub struct Weapon {
    /// Damage dealt per hit (in health points)
    pub damage: i32,

    /// Fire rate in shots per second
    #[tiled(default = 1.0)]
    pub fire_rate: f32,

    /// Maximum ammunition capacity (None = infinite)
    pub max_ammo: Option<i32>,
}
```

### Testing

Test your components work correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_defaults() {
        let mut props = tiled::Properties::new();
        props.insert("name".to_string(),
            tiled::PropertyValue::StringValue("TestPlayer".to_string()));

        // Deserialize (would normally happen via TiledClass registry)
        // Verify defaults applied correctly
    }
}
```

## Troubleshooting

### Component Not Attached

**Problem:** Object spawns but component isn't attached.

**Solutions:**
1. Verify `#[reflect(Component)]` is present
2. Check `.register_type::<YourComponent>()` is called
3. Ensure property type in Tiled matches `#[tiled(name = "...")]`
4. Check for deserialization errors in logs (warn level)

### Type Not Appearing in Tiled

**Problem:** Custom type doesn't show in Tiled dropdown.

**Solutions:**
1. Verify JSON export is configured
2. Check `exported_types.json` was generated
3. Re-import in Tiled (View → Custom Types → Import)
4. Verify type name format: `namespace::TypeName`

### Property Deserialization Fails

**Problem:** Warning: "Failed to deserialize component 'X' for property 'Y'"

**Solutions:**
1. Check type compatibility (int ↔ i32, float ↔ f32, etc.)
2. For enums, implement `FromTiledProperty`
3. Ensure required fields (no default) are set in Tiled
4. Check for type mismatches (string in int field, etc.)

### Missing ReflectComponent Error

**Problem:** "Type 'X' is registered but missing ReflectComponent"

**Solution:** Add `#[reflect(Component)]` attribute:

```rust
#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]  // ← This is required!
#[tiled(name = "...")]
pub struct MyComponent { }
```

## Advanced Topics

### Custom Deserialization

For complex types, implement `FromTiledProperty`:

```rust
impl FromTiledProperty for MyComplexType {
    fn from_property(value: &tiled::PropertyValue) -> Option<Self> {
        match value {
            tiled::PropertyValue::StringValue(s) => {
                // Parse string, return Some(value) or None
                MyComplexType::from_str(s).ok()
            }
            _ => None,
        }
    }
}
```

### Programmatic Export

Export types without file I/O (useful for testing):

```rust
use bevy_tiled_core::properties::build_export_data;

let registry = TiledClassRegistry::build();
let types = build_export_data(&registry);

for type_export in types {
    println!("Type: {}", type_export.name);
    for member in &type_export.members {
        println!("  - {}: {:?}", member.name, member.tiled_type);
    }
}
```

## See Also

- [Tiled Documentation](https://doc.mapeditor.org/)
- [Tiled Custom Properties](https://doc.mapeditor.org/en/stable/manual/custom-properties/)
- [bevy_tiled Examples](../examples/)
- [Bevy Reflection Guide](https://bevyengine.org/learn/book/reflection/)
