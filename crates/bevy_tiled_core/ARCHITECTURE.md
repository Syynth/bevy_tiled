# bevy_tiled Architecture Guide

This document explains the three-layer architecture of `bevy_tiled` and how the pieces fit together.

## The Three-Layer System

`bevy_tiled` is designed with a clear separation of concerns across three layers:

```
┌─────────────────────────────────────────────────────────┐
│ Layer 3: Extensions (Rendering, Physics, Game Logic)   │
│  - bevy_tiled_render (sprite rendering)                 │
│  - bevy_tiled_tilemap (bevy_ecs_tilemap integration)    │
│  - bevy_tiled_avian (physics colliders)                 │
│  - Your custom plugins                                  │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ Observes events
                          │ Queries components
                          │
┌─────────────────────────────────────────────────────────┐
│ Layer 2: Entity Spawning (bevy_tiled_core)             │
│  - Spawns entity hierarchy (map, layers, objects)       │
│  - Merges properties (template inheritance)             │
│  - Pre-processes data (GIDs, vertices, flip flags)      │
│  - Emits spawning events                                │
│  - Auto-attaches TiledClass components                  │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ Loads assets
                          │
┌─────────────────────────────────────────────────────────┐
│ Layer 1: Asset Loading (bevy_tiled_assets)             │
│  - Loads .tmx, .tsx, .tx files                          │
│  - Parses Tiled JSON/XML format                         │
│  - Manages asset dependencies                           │
│  - Provides TiledMapAsset, TiledTilesetAsset, etc.      │
└─────────────────────────────────────────────────────────┘
```

### Layer 1: Asset Loading (bevy_tiled_assets)

**Responsibility**: Load and parse Tiled files into Bevy assets.

**Key Types**:
- `TiledMapAsset` - Parsed .tmx file
- `TiledTilesetAsset` - Parsed .tsx file
- `TiledTemplateAsset` - Parsed .tx file

**What it does**:
- Loads files from disk/web
- Parses Tiled's JSON/XML format
- Resolves asset dependencies (embedded tilesets, external references)
- Provides readonly access to map data

**What it does NOT do**:
- Does NOT spawn entities
- Does NOT handle rendering
- Does NOT process properties

### Layer 2: Entity Spawning (bevy_tiled_core)

**Responsibility**: Convert assets into a structured ECS hierarchy.

**Key Types**:
- `TiledMap` - Component marking map entities
- `TiledLayer` - Enum for layer types (Tiles, Objects, Image, Group)
- `TiledObject` - Enum for object shapes with pre-computed vertices
- `TileLayerData` - Pre-processed tile grid
- `MergedProperties` - Component with resolved property inheritance
- Events: `TileLayerSpawned`, `ObjectSpawned`, etc.

**What it does**:
1. **Spawns entity hierarchy**:
   ```
   TiledMap Entity
   ├─ TiledLayer (Tiles) + TileLayerData
   ├─ TiledLayer (Objects)
   │  └─ TiledObject entities
   ├─ TiledLayer (Image) + ImageLayerData
   └─ TiledLayer (Group)
      └─ Child layers...
   ```

2. **Pre-processes data**:
   - Resolves GIDs → (tileset_handle, local_tile_id, flip_flags)
   - Converts object points → pre-computed vertices
   - Merges template properties into objects
   - Embeds tileset handles where needed

3. **Auto-attaches components**:
   - Finds "class" properties in objects/layers
   - Looks up registered TiledClass types
   - Deserializes and attaches components via reflection

4. **Emits events**:
   - `TileLayerSpawned` - when tile layer ready
   - `ObjectSpawned` - when object entity created
   - `ImageLayerSpawned`, `ObjectLayerSpawned`, etc.

**What it does NOT do**:
- Does NOT spawn individual tile entities (tiles are data, not entities)
- Does NOT add rendering components (Sprite, TilemapBundle, etc.)
- Does NOT add physics components (Collider, RigidBody, etc.)
- Does NOT handle gameplay logic

**Why no tile entities?**
Different rendering backends have different requirements:
- `bevy_ecs_tilemap` uses `TileStorage` with batched rendering
- Native tilemaps might use single entities with component-based tiles
- Software renderers might use image compositing

Layer 2 provides `TileLayerData` (pre-processed grid), and Layer 3 decides how to render it.

### Layer 3: Extensions (Your Plugins)

**Responsibility**: Add rendering, physics, gameplay behavior.

**Key Pattern**: Observe Layer 2 events, add components to spawned entities.

**Example Plugins**:

#### Rendering Plugin
```rust
fn on_tile_layer_spawned(
    trigger: On<TileLayerSpawned>,
    layer_query: Query<&TileLayerData>,
    mut commands: Commands,
) {
    let tile_data = layer_query.get(trigger.event().entity).unwrap();

    // Use tile_data to create bevy_ecs_tilemap TilemapBundle
    // or native Bevy sprites, or whatever you want
}
```

#### Physics Plugin
```rust
fn on_object_spawned(
    trigger: On<ObjectSpawned>,
    object_query: Query<&TiledObject>,
    mut commands: Commands,
) {
    let object = object_query.get(trigger.event().entity).unwrap();

    match object {
        TiledObject::Rectangle { width, height } => {
            // Vertices already computed!
            commands.entity(trigger.event().entity)
                .insert(Collider::cuboid(*width, *height));
        }
        TiledObject::Polygon { vertices } => {
            commands.entity(trigger.event().entity)
                .insert(Collider::convex_hull(vertices.clone()));
        }
        _ => {}
    }
}
```

**Why this design?**
- **Decoupling**: Rendering and physics don't depend on each other
- **Flexibility**: Users can mix and match plugins
- **Simplicity**: Each plugin has a single responsibility
- **Performance**: Pre-processing done once in Layer 2, used by all Layer 3 plugins

## Entity Hierarchy

### Map Entity
```rust
commands.spawn((
    TiledMap { handle: asset_server.load("map.tmx") },
    Transform::default(),
    Name::new("My Map"),
));
```

Layer 2's `process_loaded_maps()` system detects this entity, waits for assets to load, then spawns the full hierarchy.

### Relationships

Layer 2 uses Bevy's relationship system for bidirectional traversal:

```rust
// Map → Layers
#[relationship(relationship_target = LayersInMap)]
struct TiledLayerMapOf(Entity);

#[relationship_target(relationship = TiledLayerMapOf)]
struct LayersInMap(Vec<Entity>);

// Object → Map
#[relationship(relationship_target = ObjectsInMap)]
struct TiledObjectMapOf(Entity);

#[relationship_target(relationship = TiledObjectMapOf)]
struct ObjectsInMap(Vec<Entity>);

// Map → World (for multi-map support)
#[relationship(relationship_target = MapsInWorld)]
struct TiledWorldOf(Entity);

#[relationship_target(relationship = TiledWorldOf)]
struct MapsInWorld(Vec<Entity>);
```

**Usage**:
```rust
// Find all layers in a map
fn find_layers(
    map_query: Query<&LayersInMap>,
    layer_query: Query<&TiledLayer>,
) {
    for layers in map_query.iter() {
        for &layer_entity in &layers.0 {
            let layer_type = layer_query.get(layer_entity).unwrap();
            // ...
        }
    }
}

// Find the map an object belongs to
fn find_parent_map(
    object_query: Query<&TiledObjectMapOf>,
) {
    for map_of in object_query.iter() {
        let map_entity = map_of.0;
        // ...
    }
}
```

## Property System

The property system has two access patterns:

### 1. Auto-Attached Components (Recommended)

Define components with `#[derive(TiledClass)]`:

```rust
use bevy::prelude::*;
use bevy_tiled_macros::TiledClass;

#[derive(Component, Reflect, TiledClass)]
#[reflect(Component)]
#[tiled(name = "game::Enemy")]
struct Enemy {
    health: f32,
    speed: f32,
    enemy_type: String,
}
```

In Tiled:
1. Export types: Run app with `BevyTiledCorePlugin::new(BevyTiledCoreConfig { export_types_path: Some("types.json") })`
2. Import in Tiled: View → Custom Types → Import
3. Add properties: Select object → Add Property → Choose "game::Enemy" from dropdown

When map loads, Layer 2 automatically:
1. Finds the "game::Enemy" property
2. Deserializes property values
3. Attaches `Enemy` component to the entity

Access in systems:
```rust
fn enemy_ai_system(query: Query<&Enemy>) {
    for enemy in query.iter() {
        // Use enemy.health, enemy.speed, etc.
    }
}
```

### 2. MergedProperties Component (Fallback)

For untyped properties or custom logic:

```rust
fn check_properties(query: Query<&MergedProperties>) {
    for props in query.iter() {
        if let Some(damage) = props.get_f32("damage") {
            // Use raw property value
        }
    }
}
```

`MergedProperties` is always attached, even if no TiledClass components match.

### Property Merging

Layer 2 handles template inheritance automatically:

```
Template: enemy_base.tx
  health: 100
  speed: 5.0

Object: uses enemy_base.tx
  speed: 10.0  ← Overrides template

Result in MergedProperties:
  health: 100  ← From template
  speed: 10.0  ← From object
```

This happens at spawn time, so runtime queries just see the final merged result.

## Event System

Layer 2 emits observer events at key points:

### Available Events

```rust
// Emitted when a tile layer entity is spawned with TileLayerData
TileLayerSpawned { entity, map_entity, layer_id, properties }

// Emitted when an object layer entity is spawned (before individual objects)
ObjectLayerSpawned { entity, map_entity, layer_id, properties }

// Emitted for each individual object entity
ObjectSpawned { entity, map_entity, object_id, properties }

// Emitted when an image layer entity is spawned
ImageLayerSpawned { entity, map_entity, layer_id, properties }

// Emitted when a group layer entity is spawned
GroupLayerSpawned { entity, map_entity, layer_id, properties }
```

### Observer Pattern (Bevy 0.17+)

```rust
app.add_observer(on_tile_layer_spawned);

fn on_tile_layer_spawned(
    trigger: On<TileLayerSpawned>,
    layer_query: Query<&TileLayerData>,
) {
    let event = trigger.event();
    // event.entity - the layer entity
    // event.map_entity - parent map
    // event.layer_id - Tiled's layer ID
    // event.properties - merged properties

    let tile_data = layer_query.get(event.entity).unwrap();
    // Use tile_data to create rendering...
}
```

### Why Observers?

- **Decoupled**: Plugins don't know about each other
- **Composable**: Multiple plugins can observe the same event
- **Flexible**: Easy to add/remove plugin functionality
- **Performant**: Bevy's observer system is optimized

## Data Flow

### Map Loading Flow

```
1. User spawns TiledMap entity
   └─> commands.spawn(TiledMap { handle })

2. Layer 1 (assets) loads .tmx file
   └─> Parses XML/JSON
   └─> Loads embedded/external tilesets
   └─> Loads templates
   └─> Emits AssetEvent::LoadedWithDependencies

3. Layer 2 (core) detects loaded map
   └─> process_loaded_maps() system runs
   └─> For each layer:
       ├─> Tile Layer
       │   ├─> Build TileLayerData (resolve GIDs, extract flip flags)
       │   ├─> Spawn layer entity with TileLayerData
       │   ├─> Attach MergedProperties
       │   └─> Trigger TileLayerSpawned event
       │
       ├─> Object Layer
       │   ├─> Spawn layer entity
       │   ├─> For each object:
       │   │   ├─> Compute vertices for shapes
       │   │   ├─> Merge template properties
       │   │   ├─> Spawn object entity with TiledObject
       │   │   ├─> Attach MergedProperties
       │   │   ├─> Auto-attach TiledClass components
       │   │   └─> Trigger ObjectSpawned event
       │   └─> Trigger ObjectLayerSpawned event
       │
       ├─> Image Layer
       │   ├─> Load image asset
       │   ├─> Spawn layer entity with ImageLayerData
       │   └─> Trigger ImageLayerSpawned event
       │
       └─> Group Layer (recursive)
           └─> Process child layers

4. Layer 3 (extensions) reacts to events
   └─> Rendering plugin observes TileLayerSpawned
       └─> Creates TilemapBundle or Sprites
   └─> Physics plugin observes ObjectSpawned
       └─> Creates Colliders based on TiledObject shape
   └─> Custom plugin observes ObjectSpawned
       └─> Checks for specific TiledClass components
       └─> Initializes AI, spawns effects, etc.
```

### Property Flow

```
Tiled Editor
  ↓ (user adds class property "game::Enemy")
  ↓ (user sets health=50, speed=10)
  ↓
.tmx file
  <object>
    <properties>
      <property name="Enemy" type="class" propertytype="game::Enemy">
        <properties>
          <property name="health" type="float" value="50"/>
          <property name="speed" type="float" value="10"/>
        </properties>
      </property>
    </properties>
  </object>
  ↓
Layer 1: TiledMapAsset (readonly)
  object.properties["Enemy"] = ClassValue { ... }
  ↓
Layer 2: Object spawning
  ├─> Merge template properties (if template used)
  ├─> Create MergedProperties component
  ├─> Look up "game::Enemy" in TiledClassRegistry
  ├─> Call Enemy::__tiled_from_properties(&properties)
  ├─> Get Box<dyn Reflect> containing Enemy { health: 50, speed: 10 }
  ├─> Insert via ReflectComponent
  └─> Entity now has Enemy component!
  ↓
Layer 3 / Your Game Code
  Query<&Enemy>
    └─> enemy.health == 50.0
    └─> enemy.speed == 10.0
```

## Design Rationale

### Why Three Layers?

**Separation of Concerns**:
- Layer 1: File I/O and parsing (no ECS knowledge)
- Layer 2: ECS structure (no rendering/physics knowledge)
- Layer 3: Rendering/physics/gameplay (no file format knowledge)

**Flexibility**:
- Swap rendering backends without touching asset loading
- Use different physics engines without changing spawning logic
- Mix and match Layer 3 plugins

**Testability**:
- Layer 1 can be tested with sample .tmx files
- Layer 2 can be tested with mock assets
- Layer 3 can be tested by spawning entities directly

### Why Pre-Process Data?

Layer 2 pre-processes data (GIDs, vertices, properties) because:

1. **Single source of truth**: Computed once, used by all Layer 3 plugins
2. **Performance**: No redundant processing in multiple plugins
3. **Simplicity**: Layer 3 gets clean, ready-to-use data
4. **Correctness**: Complex logic (GID resolution, template merging) in one place

### Why Not Spawn Tile Entities?

Tiles are NOT spawned as individual entities because:

1. **Performance**: Thousands of tiles = thousands of entities = overhead
2. **Flexibility**: Different renderers need different structures
3. **Simplicity**: Most games don't need per-tile entities

Instead, Layer 2 provides `TileLayerData` (a component), and Layer 3 decides how to render it:
- `bevy_ecs_tilemap`: Creates optimized `TileStorage` with batched rendering
- Custom renderer: Could composite tiles into a single texture
- Debug renderer: Could spawn sprites for debugging

### Why Observers Over EventReader?

Bevy 0.17+ observers provide:
- Type-safe event handling
- Automatic cleanup
- Better composition
- Integration with Bevy's relationship system

## Common Patterns

### Pattern 1: Basic Map Spawning

```rust
fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap { handle: asset_server.load("map.tmx") },
        Transform::default(),
    ));
    // That's it! Layer 2 handles the rest automatically.
}
```

### Pattern 2: Layer 3 Rendering Plugin

```rust
pub struct MyRenderPlugin;

impl Plugin for MyRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(render_tile_layers)
           .add_observer(render_objects);
    }
}

fn render_tile_layers(
    trigger: On<TileLayerSpawned>,
    query: Query<&TileLayerData>,
    mut commands: Commands,
) {
    let tile_data = query.get(trigger.event().entity).unwrap();
    // Add rendering components...
}

fn render_objects(
    trigger: On<ObjectSpawned>,
    query: Query<&TiledObject>,
    mut commands: Commands,
) {
    let object = query.get(trigger.event().entity).unwrap();
    // Add sprite/mesh...
}
```

### Pattern 3: Conditional Component Attachment

```rust
fn on_object_spawned(
    trigger: On<ObjectSpawned>,
    query: Query<(&TiledObject, &MergedProperties)>,
    mut commands: Commands,
) {
    let (object, props) = query.get(trigger.event().entity).unwrap();

    // Only add collider if object has "collidable" property
    if props.get_bool("collidable").unwrap_or(false) {
        match object {
            TiledObject::Rectangle { width, height } => {
                commands.entity(trigger.event().entity)
                    .insert(Collider::cuboid(*width, *height));
            }
            _ => {}
        }
    }
}
```

### Pattern 4: Multi-Map Support

```rust
// Spawn multiple maps
fn spawn_maps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let world_entity = commands.spawn(Name::new("Game World")).id();

    commands.spawn((
        TiledMap { handle: asset_server.load("level1.tmx") },
        TiledWorldOf(world_entity),
        Transform::default(),
    ));

    commands.spawn((
        TiledMap { handle: asset_server.load("level2.tmx") },
        TiledWorldOf(world_entity),
        Transform::from_xyz(1000.0, 0.0, 0.0),
    ));
}

// Find all maps in a world
fn list_maps_in_world(
    world_query: Query<&MapsInWorld>,
    map_query: Query<&TiledMap>,
) {
    for maps in world_query.iter() {
        for &map_entity in &maps.0 {
            let map = map_query.get(map_entity).unwrap();
            // ...
        }
    }
}
```

## Performance Considerations

### Layer 2 Performance

**Spawning is one-time cost**:
- Happens when asset loads, not every frame
- Pre-processing (GID resolution, vertex computation) done once
- Results cached in components

**No per-frame overhead**:
- No systems run after spawning completes
- All data stored in components for fast queries

**Memory usage**:
- `TileLayerData` stores grid (width × height × TileInstance)
- `TiledObject` stores pre-computed vertices
- `MergedProperties` stores property HashMap
- All reasonable for typical maps

### Layer 3 Performance

**Use batched rendering**:
- `bevy_ecs_tilemap` for tile layers (batched draw calls)
- Sprite atlases for objects
- Mesh batching for repeated shapes

**Don't iterate tiles every frame**:
- Extract needed data in observer, store in component
- Use spatial indexing for object queries
- Cache expensive computations

## Extending bevy_tiled

### Writing a Layer 3 Plugin

1. **Choose what to extend**: Rendering? Physics? AI?

2. **Observe relevant events**:
   ```rust
   app.add_observer(my_observer);
   ```

3. **Query pre-processed data**:
   ```rust
   fn my_observer(
       trigger: On<TileLayerSpawned>,
       query: Query<&TileLayerData>,
   ) { ... }
   ```

4. **Add components**:
   ```rust
   commands.entity(event.entity).insert(MyComponent);
   ```

5. **Add update systems** (if needed):
   ```rust
   app.add_systems(Update, my_update_system);
   ```

### Contributing to bevy_tiled

**Layer 1 contributions**:
- Support for new Tiled features (.tmj format, new property types, etc.)
- Asset loader optimizations
- Better error messages

**Layer 2 contributions**:
- New pre-processing helpers
- Additional relationship types
- Performance optimizations
- More events for fine-grained control

**Layer 3 contributions** (as separate crates):
- New rendering backends
- Physics integrations
- Editor tools
- Asset hot-reloading

## Troubleshooting

### "My map isn't spawning"

1. Check asset loaded: `asset_server.get_load_state(&handle)`
2. Check console for errors
3. Verify TiledMap component exists: `Query<&TiledMap>`
4. Check if `BevyTiledCorePlugin` is added

### "Properties aren't working"

1. Export types: Set `export_types_path` in plugin config
2. Import in Tiled: View → Custom Types → Import
3. Verify #[tiled(name = "...")] matches Tiled's property type name exactly
4. Check property type (must be "class", not basic type)
5. Verify Component + Reflect derives

### "Tiles/objects render incorrectly"

This is Layer 3 (rendering) - check your rendering plugin.
Layer 2 only provides data; rendering is up to Layer 3.

### "Performance is slow"

1. Check tile count - very large maps may need chunking
2. Use bevy_ecs_tilemap for tile layers (not individual sprites)
3. Profile with `cargo install cargo-flamegraph`
4. Check for per-frame iteration over tiles

## Summary

The three-layer architecture provides:

✅ **Clean separation**: Assets, spawning, rendering are independent
✅ **Flexibility**: Mix and match rendering/physics backends
✅ **Simplicity**: Each layer has a single, well-defined job
✅ **Extensibility**: Easy to add new functionality via Layer 3 plugins
✅ **Performance**: Pre-processing done once, optimal data structures

**Layer 1** loads files, **Layer 2** spawns entities, **Layer 3** adds behavior.

See `extension_pattern.rs` example for a complete demonstration.
