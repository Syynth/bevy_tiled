# Custom Properties System

The custom properties system enables Tiled custom properties to become Bevy components, with proper type safety and inheritance.

---

## Overview

**Tiled Editor** allows adding custom properties to:
- Maps
- Layers
- Tiles (in tilesets)
- Templates
- Objects (in maps)

**bevy_tiled_topdown** converts these properties into **Bevy components** using reflection.

---

## Property Flow

```
Tiled Editor
  ↓ (defines custom classes)
  ↓
JSON Export ──────────────────────────┐
  (propertyTypes.json)                 │
  ↓                                    │
Tiled .tmx/.tsx/.tx files              │
  (properties with values)             │
  ↓                                    │
LAYER 1: Asset Loading                 │
  • Parse property VALUES              │
  • Store in DeserializedProperties    │
  • NO inheritance yet                 │
  ↓                                    │
LAYER 2: Entity Spawning               │
  • Merge properties (inheritance)     │
  • Convert to Bevy components         │
  • Insert on entities                 │
  ↓                                    │
LAYER 3: Reactive Systems              │
  • Query entities by components       │
  • React to component values          │
                                       │
Bevy App (Rust)                        │
  ↓ (reflection-based export)          │
  ↓                                    │
JSON Export ←──────────────────────────┘
  (for Tiled editor import)
```

---

## Defining Custom Properties

### Step 1: Define Components in Rust

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Clone, Debug, Deserialize, Serialize)]
#[reflect(Component)]
pub struct Door {
    pub target_map: String,
    pub target_position: IVec2,
    pub locked: bool,
}

#[derive(Component, Reflect, Clone, Debug, Deserialize, Serialize)]
#[reflect(Component)]
pub struct SpawnPoint {
    pub player_type: PlayerType,
    pub facing: FacingDirection,
}

#[derive(Reflect, Clone, Debug, Deserialize, Serialize)]
pub enum PlayerType {
    Hero,
    Companion,
}

#[derive(Reflect, Clone, Debug, Deserialize, Serialize)]
pub enum FacingDirection {
    North,
    South,
    East,
    West,
}
```

### Step 2: Register Types in Bevy

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TiledPlugin(TiledPluginConfig {
            tiled_types_export_file: Some(PathBuf::from("assets/editor/custom_properties.json")),
            tiled_types_filter: TiledFilter::RegexSet(
                RegexSet::new(["myapp::.*"]).unwrap()
            ),
        }))
        .register_type::<Door>()
        .register_type::<SpawnPoint>()
        .register_type::<PlayerType>()
        .register_type::<FacingDirection>()
        .run();
}
```

### Step 3: Export for Tiled Editor

On app startup, `bevy_tiled_topdown` exports registered types to JSON:

```json
{
  "classes": [
    {
      "id": 1,
      "name": "myapp::Door",
      "type": "class",
      "members": [
        { "name": "target_map", "type": "string" },
        { "name": "target_position", "type": "class", "propertyType": "glam::IVec2" },
        { "name": "locked", "type": "bool" }
      ]
    },
    {
      "id": 2,
      "name": "myapp::SpawnPoint",
      "type": "class",
      "members": [
        { "name": "player_type", "type": "class", "propertyType": "myapp::PlayerType" },
        { "name": "facing", "type": "class", "propertyType": "myapp::FacingDirection" }
      ]
    },
    {
      "id": 3,
      "name": "myapp::PlayerType",
      "type": "enum",
      "values": ["Hero", "Companion"],
      "valuesAsFlags": false
    }
  ]
}
```

### Step 4: Import in Tiled Editor

1. Open Tiled
2. **View → Custom Types Editor**
3. Click **Import**
4. Select `assets/editor/custom_properties.json`
5. Custom classes now available in Tiled

### Step 5: Use in Tiled

Create an object in Tiled:
1. Add an object to a layer
2. In **Properties** panel, click **+** → **Add Property**
3. Select type: **myapp::Door**
4. Set values:
   - `target_map`: "overworld.tmx"
   - `target_position.x`: 10
   - `target_position.y`: 5
   - `locked`: true

---

## Property Inheritance (Spawn Time)

**CRITICAL:** Inheritance happens during entity spawning, not asset loading.

### Inheritance Order (Priority)

```
Tile Properties       ← Lowest priority (base)
  ↓ (overridden by)
Template Properties   ← Middle priority
  ↓ (overridden by)
Object Properties     ← Highest priority (wins conflicts)
```

### Example Scenario

**Tileset tile_id=42:**
```xml
<tile id="42">
  <properties>
    <property name="enemy" type="class" propertytype="myapp::Enemy">
      <property name="health" type="int" value="100"/>
      <property name="damage" type="int" value="10"/>
      <property name="armor" type="int" value="5"/>
    </property>
  </properties>
</tile>
```

**Template "goblin.tx":**
```xml
<template>
  <object gid="42" width="32" height="32">
    <properties>
      <property name="enemy" type="class" propertytype="myapp::Enemy">
        <property name="damage" type="int" value="15"/>
        <property name="speed" type="float" value="2.5"/>
      </property>
    </properties>
  </object>
</template>
```

**Object in map (uses template):**
```xml
<object template="goblin.tx" x="100" y="200">
  <properties>
    <property name="enemy" type="class" propertytype="myapp::Enemy">
      <property name="armor" type="int" value="10"/>
    </property>
  </properties>
</object>
```

### Asset Layer (No Inheritance)

Assets store **raw, unmerged** properties:

```rust
// TiledTilesetAsset
tile_properties: {
    42: {
        "enemy": Enemy { health: 100, damage: 10, armor: 5 }
    }
}

// TiledTemplateAsset
properties: {
    "enemy": Enemy { damage: 15, speed: Some(2.5) }
}

// TiledMapAsset
properties.objects: {
    7: {
        "enemy": Enemy { armor: 10 }
    }
}
```

### Spawning Layer (Inheritance)

```rust
fn spawn_object_with_properties(
    object: &tiled::Object,
    tileset_asset: Option<&TiledTilesetAsset>,
    template_asset: Option<&TiledTemplateAsset>,
    map_asset: &TiledMapAsset,
) {
    // Start empty
    let mut merged = DeserializedProperties::default();

    // Step 1: Merge tile properties
    if let Some(tile_props) = get_tile_properties(...) {
        merged.merge_from(tile_props);
    }
    // merged = { enemy: { health: 100, damage: 10, armor: 5 } }

    // Step 2: Merge template properties
    if let Some(template_props) = template_asset.properties {
        merged.merge_from(template_props);
    }
    // merged = {
    //   enemy: {
    //     health: 100,       ← from tile
    //     damage: 15,        ← overridden by template
    //     armor: 5,          ← from tile
    //     speed: Some(2.5)   ← added by template
    //   }
    // }

    // Step 3: Merge object properties
    if let Some(object_props) = map_asset.properties.objects.get(&object.id) {
        merged.merge_from(object_props);
    }
    // merged = {
    //   enemy: {
    //     health: 100,       ← from tile
    //     damage: 15,        ← from template
    //     armor: 10,         ← overridden by object
    //     speed: Some(2.5)   ← from template
    //   }
    // }

    // Step 4: Convert to components
    let bundle = merged.to_bundle(registry);
    commands.entity(entity).insert(bundle);
}
```

### Final Entity Components

```rust
Entity {
    Enemy {
        health: 100,       // from tile
        damage: 15,        // from template
        armor: 10,         // from object instance
        speed: Some(2.5),  // from template
    },
    TiledObject::Tile { width: 32.0, height: 32.0 },
    Transform::from_xyz(100.0, 200.0, 0.0),
    // ...
}
```

---

## Merge Algorithm

### DeserializedProperties::merge_from()

```rust
impl DeserializedProperties {
    /// Merge properties from another set, with `other` taking priority
    pub fn merge_from(&mut self, other: &DeserializedProperties) {
        for (key, value) in &other.properties {
            match (self.properties.get_mut(key), value) {
                // Both are structs → recursively merge fields
                (Some(PropertyValue::Struct(existing)), PropertyValue::Struct(incoming)) => {
                    existing.merge_from(incoming);
                }

                // Other cases → incoming value replaces existing
                _ => {
                    self.properties.insert(key.clone(), value.clone());
                }
            }
        }
    }
}
```

### Merge Rules

1. **New property:** Add to merged result
2. **Primitive override:** Newer value replaces older value
3. **Struct merge:** Recursively merge struct fields
4. **Enum override:** Newer enum value replaces older value
5. **Array override:** Newer array replaces older array (no element-wise merging)

---

## File Properties (Handles)

### The Problem

Tiled file properties use **relative paths**:

```xml
<property name="transition" type="file" value="../transitions/fade.transition.toml"/>
```

These paths are:
- Relative to the TMX/TSX/TX file
- Just strings in Tiled
- Need to become `Handle<T>` in Bevy

### Solution: Convert During Asset Loading

```rust
// In TiledMapAssetLoader::load()
fn parse_file_property(
    property: &tiled::PropertyValue,
    load_context: &mut LoadContext,
) -> PropertyValue {
    match property {
        tiled::PropertyValue::FileValue(relative_path) => {
            // Get the directory containing this asset
            let asset_dir = load_context.path().parent().unwrap();

            // Join relative path
            let full_path = asset_dir.join(relative_path);

            // Load as dependency
            let handle: Handle<TransitionAsset> = load_context.load(full_path);

            // Store handle (not string) in properties
            PropertyValue::Handle(handle)
        }
        _ => { /* ... */ }
    }
}
```

### Example

**Tiled file:** `assets/maps/town.tmx`

**Property in Tiled:**
```xml
<property name="transition" type="file" value="../transitions/fade.transition.toml"/>
```

**Asset loading:**
```rust
// load_context.path() = "maps/town.tmx"
// asset_dir = "maps"
// relative_path = "../transitions/fade.transition.toml"
// full_path = "transitions/fade.transition.toml"

let handle: Handle<TransitionAsset> = load_context.load("transitions/fade.transition.toml");
```

**Component on entity:**
```rust
#[derive(Component, Reflect)]
pub struct TransitionReference {
    pub transition: Handle<TransitionAsset>,
}

// Entity gets this component:
TransitionReference {
    transition: Handle<TransitionAsset> { /* ... */ }
}
```

This **fixes the relative path bug** in `bevy_ecs_tiled`.

---

## Supported Property Types

### Primitives

| Tiled Type | Rust Type |
|------------|-----------|
| `bool` | `bool` |
| `int` | `i32` |
| `float` | `f32` |
| `string` | `String` |
| `color` | `bevy::Color` |

### Bevy Types

| Tiled Type | Rust Type |
|------------|-----------|
| `file` | `Handle<T>` (asset handle) |
| `object` | `Entity` (after hydration) |

### Custom Types

| Tiled Type | Rust Type |
|------------|-----------|
| `class` (struct) | Any `#[derive(Reflect)]` struct |
| `enum` | Any `#[derive(Reflect)]` enum |

### Collections

| Tiled Type | Rust Type |
|------------|-----------|
| `array` | `Vec<T>` |

### Example Struct

**Tiled:**
```xml
<property name="spawn_target" type="class" propertytype="myapp::SpawnTarget">
  <property name="target_map" type="string" value="overworld.tmx"/>
  <property name="target_position" type="class" propertytype="glam::IVec2">
    <property name="x" type="int" value="10"/>
    <property name="y" type="int" value="5"/>
  </property>
  <property name="facing" type="class" propertytype="myapp::FacingDirection">
    <property name=":variant" value="North"/>
  </property>
</property>
```

**Rust:**
```rust
#[derive(Component, Reflect)]
pub struct SpawnTarget {
    pub target_map: String,
    pub target_position: IVec2,
    pub facing: FacingDirection,
}

// Entity component:
SpawnTarget {
    target_map: "overworld.tmx".into(),
    target_position: IVec2::new(10, 5),
    facing: FacingDirection::North,
}
```

---

## Entity References (Object Properties)

### The Problem

Tiled objects can reference **other objects** by ID:

```xml
<object id="1" name="switch" x="100" y="200">
  <properties>
    <property name="target_door" type="object" value="42"/>
  </properties>
</object>

<object id="42" name="door" x="300" y="400">
  <!-- ... -->
</object>
```

Object `1` references object `42`. But we need a Bevy `Entity`, not a Tiled ID.

### Solution: Hydration

**Asset loading:**
```rust
// Store the Tiled object ID (u32)
properties.objects.insert(1, {
    "target_door": PropertyValue::TiledObjectId(42)
});
```

**Entity spawning:**
```rust
// After all objects are spawned, "hydrate" the references
fn hydrate_object_references(
    properties: &mut DeserializedProperties,
    storage: &TiledMapStorage,
) {
    for (key, value) in &mut properties.properties {
        if let PropertyValue::TiledObjectId(object_id) = value {
            // Look up entity for this object ID
            if let Some(entity) = storage.get_object_entity(*object_id) {
                *value = PropertyValue::Entity(entity);
            }
        }
    }
}
```

**Component on entity:**
```rust
#[derive(Component, Reflect)]
pub struct Switch {
    pub target_door: Entity,  // ← Bevy entity, not Tiled ID
}

// Entity gets:
Switch {
    target_door: Entity::from_raw(123)  // ← Actual Bevy entity
}
```

Now you can query the target:

```rust
fn switch_activated(
    q_switches: Query<&Switch>,
    mut q_doors: Query<&mut Door>,
) {
    for switch in q_switches.iter() {
        if let Ok(mut door) = q_doors.get_mut(switch.target_door) {
            door.locked = false;
        }
    }
}
```

---

## Using Custom Properties

### Querying Entities

```rust
fn door_system(
    q_doors: Query<(Entity, &Door, &Transform), With<TiledObject>>,
) {
    for (entity, door, transform) in q_doors.iter() {
        if door.locked {
            // Draw locked icon
        }
    }
}
```

### Reacting to Creation

```rust
fn on_door_created(
    trigger: On<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    q_doors: Query<&Door>,
) {
    let Ok(door) = q_doors.get(trigger.entity) else { return };

    // Add physics collider
    commands.entity(trigger.entity).insert((
        Sensor,
        Collider::rectangle(32.0, 32.0),
    ));

    if door.locked {
        // Add locked sprite overlay
        commands.entity(trigger.entity).with_children(|parent| {
            parent.spawn((
                Sprite { /* locked icon */ },
                Transform::from_xyz(0.0, 16.0, 1.0),
            ));
        });
    }
}

fn setup(app: &mut App) {
    app.add_observer(on_door_created);
}
```

---

## Advanced: Partial Properties

### The Challenge

Sometimes you want to specify **only some fields** of a struct:

**Template:**
```xml
<property name="enemy" type="class">
  <property name="health" type="int" value="100"/>
  <!-- damage not specified -->
</property>
```

**Object instance:**
```xml
<property name="enemy" type="class">
  <property name="damage" type="int" value="20"/>
  <!-- health not specified -->
</property>
```

### Solution: Option<T> Fields

```rust
#[derive(Component, Reflect, Default)]
pub struct Enemy {
    pub health: Option<i32>,
    pub damage: Option<i32>,
    pub armor: Option<i32>,
}
```

**Template:**
```rust
Enemy {
    health: Some(100),
    damage: None,
    armor: None,
}
```

**Merged:**
```rust
Enemy {
    health: Some(100),  // from template
    damage: Some(20),   // from object
    armor: None,        // not specified anywhere
}
```

**At runtime, use defaults:**
```rust
fn enemy_system(q_enemies: Query<&Enemy>) {
    for enemy in q_enemies.iter() {
        let health = enemy.health.unwrap_or(100);
        let damage = enemy.damage.unwrap_or(10);
    }
}
```

---

## Debugging Properties

### Viewing Raw Properties

```rust
fn debug_properties(
    trigger: On<TiledEvent<ObjectCreated>>,
    map_assets: Res<Assets<TiledMapAsset>>,
) {
    let Some(map_asset) = trigger.get_map_asset(&map_assets) else { return };

    if let Some(object_id) = trigger.get_object_id() {
        if let Some(props) = map_asset.properties.objects.get(&object_id) {
            println!("Object {} raw properties: {:?}", object_id, props);
        }
    }
}
```

### Inspecting Components

```rust
fn debug_components(world: &World, entity: Entity) {
    let registry = world.resource::<AppTypeRegistry>().read();

    for component_id in world.entity(entity).archetype().components() {
        let info = world.components().get_info(component_id).unwrap();
        println!("Component: {}", info.name());

        if let Some(type_id) = info.type_id() {
            if let Some(registration) = registry.get(type_id) {
                // Print component data using reflection
                println!("{:?}", registration);
            }
        }
    }
}
```

---

## Summary

1. **Define components** in Rust with `#[derive(Reflect)]`
2. **Export to JSON** for Tiled editor import
3. **Use in Tiled** by adding custom class properties
4. **Asset loading** parses property VALUES (no inheritance)
5. **Entity spawning** performs inheritance: tile → template → object
6. **Components inserted** on entities with final merged values
7. **Query/react** using standard Bevy ECS patterns
