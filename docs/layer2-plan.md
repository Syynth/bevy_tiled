# Implementation Plan: Layer 2 (bevy_tiled_core) - Entity Spawning Backbone

## Executive Summary

Layer 2 (bevy_tiled_core) is the **entity spawning backbone** for bevy_tiled. It converts loaded Tiled assets into a structured ECS hierarchy with property merging, ID tracking, and extension hooks. **It does NOT handle rendering or physics** - those are Layer 3+ concerns that plug in via events and component queries.

## Core Principles

1. **Backbone, Not Rendering**: Layer 2 spawns entities with data, Layer 3+ adds visuals/physics
2. **Extension Hooks**: Event system allows users to plug in custom rendering/physics/logic
3. **Property Merging**: Implements Tile → Template → Object inheritance chain
4. **Separate Crate**: bevy_tiled_core depends on bevy_tiled_assets

## Entity Hierarchy

```
TiledMap Entity (root)
├─ TiledLayer Entity (Tile Layer)
│  ├─ TiledTilemap Entity (per tileset)
│  │  └─ TiledTile Entities (individual tiles)
│  └─ ...
├─ TiledLayer Entity (Object Layer)
│  └─ TiledObject Entities
├─ TiledLayer Entity (Image Layer)
│  └─ TiledImage Entity
└─ TiledLayer Entity (Group Layer - recursive)
```

>> ~ note: we should show the way group layers are parents of their children entities. similarly, we should consider more thoroughly leveraging bevy's relationship system to provide access to the relationships between entities. something like `TiledMapOf()` which all tiled layers, objects, etc. can have. `TiledWorldOf()` could be the same but point to the world rather than the map. there are a variety of considerations here and i don't think just doing literally this is the right call, but let's discuss.

## Crate Structure

```
bevy_tiled_core/
├── src/
│   ├── lib.rs                    # Public API
│   ├── plugin.rs                 # BevyTiledCorePlugin
│   │
│   ├── components/
│   │   ├── mod.rs
│   │   ├── map.rs               # TiledMap, TiledMapStorage, TiledMapReference
│   │   ├── layer.rs             # TiledLayer enum, LayerId
│   │   ├── tile.rs              # TiledTile, TiledTilemap, TileData
│   │   ├── object.rs            # TiledObject enum, ObjectId
│   │   └── properties.rs        # MergedProperties
│   │
│   ├── spawn/
│   │   ├── mod.rs
│   │   ├── map.rs               # spawn_map() entry point
│   │   ├── layers.rs            # spawn_layers() recursive
│   │   ├── tiles.rs             # spawn_tiles_layer()
│   │   ├── objects.rs           # spawn_objects_layer()
│   │   └── images.rs            # spawn_image_layer()
│   │
│   ├── properties/
│   │   ├── mod.rs
│   │   ├── merge.rs             # Property merging logic
│   │   └── reflect.rs           # Reflection-based insertion
│   │
│   ├── events/
│   │   ├── mod.rs
│   │   └── events.rs            # TiledEvent<E>, MapCreated, etc.
│   │
│   ├── storage/
│   │   └── map_storage.rs       # TiledMapStorage HashMap lookups
│   │
│   └── systems/
│       └── spawn.rs             # process_loaded_maps() reactive system
│
├── examples/
│   ├── basic_spawn.rs           # Simple map spawning
│   ├── property_access.rs       # Accessing merged properties
│   └── extension_pattern.rs    # Layer 3 extension example
│
└── Cargo.toml
```

## Implementation Phases

### Phase 1: Core Infrastructure ✅ FIRST
**Files**: components/map.rs, components/layer.rs, plugin.rs, systems/spawn.rs

**Goal**: Basic entity spawning without properties

1. Create bevy_tiled_core crate with Cargo.toml
2. Define core components:
   - `TiledMap` - marker with map asset handle
   - `TiledMapStorage` - HashMap<ID, Entity> lookups >> ~ explain what function this serves? presumably it's coming from bevy_ecs_tilemap but i don't understand the purpose.
   - `TiledMapReference` - backreference to map entity >> ~ this could be the relationship mentioned above.
   - `TiledLayer` enum - Tiles/Objects/Image/Group
3. Implement `process_loaded_maps()` reactive system >> ~ we need to expand on what this is and what use cases it serves. do we need traits or observers or something to allow the layer 3 plugins to work properly? i know for sure i'd like a read-only version of all the data that's loaded in layer 1 as part of the current tree being spawned to be reachable from here.
4. Basic map spawning with layer hierarchy (no tiles/objects yet)
5. Register plugin and types

**Deliverable**: Map entity spawns with layer hierarchy

### Phase 2: Tile & Object Spawning
**Files**: spawn/tiles.rs, spawn/objects.rs, spawn/images.rs, components/tile.rs, components/object.rs

**Goal**: Complete entity hierarchy

1. Define tile/object components:
   - `TiledTile`, `TiledTilemap`, `TileData`
   - `TiledObject` enum (Point, Rectangle, Ellipse, Polygon, Tile, etc.)
   - `ObjectId`, `TilesetReference`
2. Implement tile layer spawning
   - Split by tileset (one TiledTilemap per tileset per layer)
   - Spawn TiledTile entities with TileData
3. Implement object layer spawning
   - Parse object shapes
   - Convert to TiledObject enum
   - Calculate transforms
4. Implement image layer spawning
5. Implement group layer recursion

>> ~ i really want to emphasize the importance of striking a balance between being reasonably performant and exposing the data we need here in a sane way. we can store shape data, etc. in a component on the entity that has it to make it easy to access and use, but it needs to be passed to whatever hooks are going to use it as well. perhaps we'll need to revisit where this data is processed so it doesn't need to be computed 1000 times in the hooks during spawning when it could be handled properly in the background thread of the asset server at startup.

**Deliverable**: Full entity hierarchy from .tmx files

### Phase 3: Property System
**Files**: properties/merge.rs, components/properties.rs

**Goal**: Property merging and storage

1. Define `MergedProperties` component
2. Implement property merging logic:
   - Tile: Tileset tile properties (no per-instance properties)
   - Object: Object properties → Template properties
3. Add `MergedProperties` to spawned entities
4. Expose property access API

>> ~ there's not enough here to critique really, but the comments about considering traits or derive macros or something to register which types can be exported and such (a la bevy_ecs_tiled) is worth considering. i don't like the system for exporting properties provided by bevy_ecs_tiled, so perhaps something like the `inventory` crate + a derive macro for registering public types would be better. then we can set a well-defined stable, user-friendly name for what shows up in the tiled custom property dropdown.

**Deliverable**: Properties accessible on spawned entities

### Phase 4: Event System
**Files**: events/events.rs

**Goal**: Extension hooks for Layer 3

1. Define `TiledEvent<E>` generic wrapper
2. Define event types: `MapCreated`, `LayerCreated`, `TileCreated`, `ObjectCreated`
3. Emit events during spawning
4. Context tracking (map, layer, tile, object entities)
5. Event transmutation (preserving context)

>> ~ this mostly seems fine, but the same concern remains above about making it ergonomic to access whatever data is needed at the time that we're spawning entities and inserting components.

**Deliverable**: Events for Layer 3 to observe

### Phase 5: Reflection Integration (Optional - Can Defer)
**Files**: properties/reflect.rs

**Goal**: Dynamic component insertion

1. Property → Component deserialization
2. Use `ReflectComponent` / `ReflectBundle`
3. Type registry integration
4. ClassValue → Reflected instance conversion

>> ~ this can't be "deferred", it's absolutely a necessary feature for this to be a useful library for me. but we might have to get further on the earlier phases before making a decision. it feels like this is getting too far down the road already.

**Deliverable**: Custom components from Tiled properties

### Phase 6: Examples & Documentation
**Files**: examples/*, README.md

**Goal**: Usability and discoverability

1. basic_spawn.rs - spawn map and iterate entities
2. property_access.rs - read merged properties
3. extension_pattern.rs - Layer 3 rendering plugin example
4. API documentation
5. Architecture guide

>> ~ can't wait!

**Deliverable**: Production-ready crate

## Key Components

### TiledMap (map.rs)
```rust
#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
pub struct TiledMap {
    pub handle: Handle<TiledMapAsset>,
}
```

### Relationship Components (map.rs)
```rust
// Layer → Map relationship
#[derive(Component)]
#[relationship(relationship_target = LayersInMap)]
pub struct TiledLayerMapOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TiledLayerMapOf)]
pub struct LayersInMap(pub Vec<Entity>);

// Object → Map relationship
#[derive(Component)]
#[relationship(relationship_target = ObjectsInMap)]
pub struct TiledObjectMapOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TiledObjectMapOf)]
pub struct ObjectsInMap(pub Vec<Entity>);

// Map → World relationship
#[derive(Component)]
#[relationship(relationship_target = MapsInWorld)]
pub struct TiledWorldOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TiledWorldOf)]
pub struct MapsInWorld(pub Vec<Entity>);
```

### TiledLayer (layer.rs)
```rust
#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
pub enum TiledLayer {
    Tiles,
    Objects,
    Image,
    Group,
}

#[derive(Component, Reflect)]
pub struct LayerId(pub u32);  // Tiled's original layer ID
```

### TileLayerData (layer.rs)
```rust
/// Raw tile grid data attached to tile layer entities.
/// Layer 3 rendering plugins decide how to render this.
#[derive(Component, Reflect)]
pub struct TileLayerData {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Option<TileInstance>>,  // Flattened [y * width + x]
}

impl TileLayerData {
    pub fn get(&self, x: u32, y: u32) -> Option<&TileInstance> { /* ... */ }
    pub fn iter_tiles(&self) -> impl Iterator<Item = (u32, u32, &TileInstance)> { /* ... */ }
}
```

### TileInstance (tile.rs)
```rust
/// Pre-processed tile data (NOT a component, stored in TileLayerData).
#[derive(Clone, Debug, Reflect)]
pub struct TileInstance {
    pub gid: u32,                                // Original GID
    pub tileset_handle: Handle<TiledTilesetAsset>,  // Which tileset
    pub tile_id: u32,                            // Local tile ID
    pub flipped_h: bool,
    pub flipped_v: bool,
    pub flipped_d: bool,
}
```

### ImageLayerData (layer.rs)
```rust
#[derive(Component, Reflect)]
pub struct ImageLayerData {
    pub image_handle: Handle<Image>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}
```

### TiledObject (object.rs)
```rust
/// Component attached to individual object entities.
/// Vertices are PRE-COMPUTED (not raw points).
#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
pub enum TiledObject {
    Point,
    Rectangle { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polygon { vertices: Vec<Vec2> },        // Pre-computed!
    Polyline { vertices: Vec<Vec2> },       // Pre-computed!
    Tile {
        tile_id: u32,
        tileset_handle: Handle<TiledTilesetAsset>,
        width: f32,
        height: f32,
    },
    Text { /* ... */ },
}

#[derive(Component, Reflect)]
pub struct ObjectId(pub u32);  // Tiled's original object ID
```

### MergedProperties (properties.rs)
```rust
#[derive(Component, Reflect)]
pub struct MergedProperties {
    properties: Properties,  // Pre-merged (object + template, or layer)
}

impl MergedProperties {
    pub fn get_float(&self, key: &str) -> Option<f32> { /* ... */ }
    pub fn get_bool(&self, key: &str) -> Option<bool> { /* ... */ }
    pub fn get_string(&self, key: &str) -> Option<&str> { /* ... */ }
    // ... accessor methods
}
```

>> ~ not sure if this is really how this should work, it might need to have some references internally with weird lifetimes and such, and then accessor methods or something. i don't _want_ that, but i suspect it'll be necessary to get the API i want. if anything, we can have the version that handles this and pre-compute the component version? the thing is that the "properties" here are actually components, so just sticking them in the Properties isn't actually useful. layer 3 and beyond shouldn't be dealing with or thinking about the raw types in the `tiled` crate. we're trying to take the data/asset management burden off the layer 3 consumers and instead make writing integrations easier.

## Property Merging Strategy

**Tile Properties**:
- Tiles don't have per-instance properties
- Use tileset-level tile definition properties
- No merging needed (just copy from tileset)

**Object Properties**:
1. Start with template properties (if template used)
2. Override with object's own properties
3. Store merged result in `MergedProperties` component

**Timing**: Merge at spawn time (pre-computed, simpler access) >> ~ again, i think this is kind of a bad idea pending a strong argument otherwise, because it's probably just not all stuff that can/should be in a single component

## Event System Pattern

```rust
#[derive(Event)]
pub struct TiledEvent<E> {
    pub entity: Entity,      // The spawned entity
    pub map_entity: Entity,  // Parent map
    pub event: E,            // Event type marker
}

// Event type markers:
pub struct MapCreated;
pub struct TileLayerCreated;    // Tile data ready for rendering
pub struct ObjectLayerCreated;
pub struct ImageLayerCreated;
pub struct GroupLayerCreated;
pub struct ObjectCreated;       // Individual object spawned
// NO TileCreated - tiles aren't entities
```

**Usage** (Layer 3 rendering):
```rust
fn spawn_tilemap_rendering(
    trigger: On<TiledEvent<TileLayerCreated>>,
    layer_query: Query<&TileLayerData>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let tile_data = layer_query.get(trigger.entity).unwrap();

    // Layer 3 has full control - example using bevy_ecs_tilemap
    let mut tile_storage = TileStorage::empty(
        TilemapSize::new(tile_data.width, tile_data.height)
    );

    for (x, y, tile) in tile_data.iter_tiles() {
        let tileset = tileset_assets.get(&tile.tileset_handle).unwrap();

        let tile_entity = commands.spawn(TileBundle {
            position: TilePos::new(x, y),
            texture_index: TileTextureIndex(tile.tile_id),
            flip: TileFlip {
                x: tile.flipped_h,
                y: tile.flipped_v,
                d: tile.flipped_d,
            },
            ..default()
        }).id();

        tile_storage.set(&TilePos::new(x, y), tile_entity);
    }

    commands.entity(trigger.entity).insert(TilemapBundle {
        storage: tile_storage,
        texture: TilemapTexture::Single(tileset.atlas_image.clone().unwrap()),
        // ...
    });
}
```

**Usage** (Layer 3 physics):
```rust
fn add_colliders_to_objects(
    trigger: On<TiledEvent<ObjectCreated>>,
    object_query: Query<(&TiledObject, &MergedProperties)>,
    mut commands: Commands,
) {
    let (object, props) = object_query.get(trigger.entity).unwrap();

    // Vertices already pre-computed in component!
    match object {
        TiledObject::Polygon { vertices } => {
            let friction = props.get_float("friction").unwrap_or(0.5);
            commands.entity(trigger.entity).insert((
                Collider::convex_hull(vertices.clone()),
                Friction::new(friction),
            ));
        }
        _ => {}
    }
}
```

>> ~ bevy 0.17 moved those to MessageReader, not EventReader, but even so we probably want to consider also supporting an observer based API as well. for high volume events like TileCreated i guess it's better to use this Bulk API like this, but for map/world created, observers are much nicer.

## Reactive Spawning System

```rust
pub fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    maps: Res<Assets<TiledMapAsset>>,
    mut commands: Commands,
    mut map_query: Query<
        (Entity, &TiledMap, &mut TiledMapStorage),
        Or<(Changed<TiledMap>, With<RespawnTiledMap>)>
    >,
) {
    for (entity, tiled_map, mut storage) in map_query.iter_mut() {
        // Check if assets finished loading
        let load_state = asset_server
            .get_recursive_dependency_load_state(&tiled_map.handle);

        if load_state.is_loaded() {
            // Spawn map hierarchy
            spawn::spawn_map(&mut commands, entity, map_asset, &mut storage);
        }
    }
}
```

**Schedule**: PreUpdate (before user systems)

## Extension Pattern (Layer 3)

### Rendering Plugin Example

```rust
// bevy_tiled_render crate
pub struct TiledRenderPlugin;

impl Plugin for TiledRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_sprites_to_tiles);
    }
}

fn add_sprites_to_tiles(
    mut commands: Commands,
    tile_query: Query<(Entity, &TileData, &TilesetReference), Added<TiledTile>>,
    tilesets: Res<Assets<TiledTilesetAsset>>,
) {
    for (entity, tile_data, tileset_ref) in tile_query.iter() {
        let Some(tileset) = tilesets.get(&tileset_ref.handle) else { continue };

        // Add Sprite based on tile data
        commands.entity(entity).insert(SpriteBundle {
            texture: tileset.atlas_image.clone().unwrap(),
            // ... configure sprite
        });
    }
}
```

### Physics Plugin Example

```rust
// bevy_tiled_physics crate
fn add_colliders_to_objects(
    mut commands: Commands,
    object_query: Query<(Entity, &TiledObject), Added<TiledObject>>,
) {
    for (entity, object) in object_query.iter() {
        match object {
            TiledObject::Rectangle { width, height } => {
                commands.entity(entity).insert(Collider::cuboid(*width, *height));
            }
            _ => {}
        }
    }
}
```

## Success Criteria

✅ Map entities spawn with layer hierarchy
✅ Tile/object/image entities created correctly
✅ Properties merged and accessible
✅ Events emitted for Layer 3 hooks
✅ ID → Entity bidirectional lookups working
✅ Examples demonstrate basic usage and extension
✅ No rendering in Layer 2 (pure data/structure)

## Critical Files (Implementation Order)

1. **src/components/map.rs** - TiledMap, TiledMapStorage (foundation)
2. **src/systems/spawn.rs** - process_loaded_maps() reactive system
3. **src/spawn/map.rs** - spawn_map() entry point
4. **src/spawn/layers.rs** - spawn_layers() recursive logic
5. **src/components/layer.rs** - TiledLayer enum
6. **src/components/tile.rs** - Tile components
7. **src/spawn/tiles.rs** - Tile spawning
8. **src/components/object.rs** - Object components
9. **src/spawn/objects.rs** - Object spawning
10. **src/events/events.rs** - Event system
11. **src/properties/merge.rs** - Property merging
12. **src/plugin.rs** - Plugin registration

## Design Decisions

**Q: When to merge properties?**
A: At spawn time. Pre-computed, simpler access, no runtime overhead.

**Q: Store in component or on-demand?**
A: Component (`MergedProperties`). Enables queries, reflection insertion, consistent access.

**Q: Events or Observers?**
A: Both. Events for now (Bevy 0.17), Observers when Bevy 0.18+ is stable.

**Q: Separate crate?**
A: Yes (bevy_tiled_core). Clean separation, users choose which layers they need.

**Q: Include rendering?**
A: No. Layer 2 is structure/data only. Rendering is Layer 3 (pluggable).

## Example Usage

### Basic Spawning
```rust
use bevy::prelude::*;
use bevy_tiled_assets::prelude::*;
use bevy_tiled_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyTiledAssetsPlugin)
        .add_plugins(BevyTiledCorePlugin)
        .add_systems(Startup, spawn_map)
        .run();
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TiledMap {
        handle: asset_server.load("maps/level1.tmx"),
    });
}
```

### Layer Access Example
```rust
fn read_layer_properties(
    layer_query: Query<(&LayerId, &MergedProperties), With<TiledLayer>>,
) {
    for (layer_id, props) in layer_query.iter() {
        if let Some(has_collision) = props.get_bool("has_collision") {
            println!("Layer {}: has_collision = {}", layer_id.0, has_collision);
        }
    }
}

fn read_object_properties(
    object_query: Query<(&ObjectId, &TiledObject, &MergedProperties)>,
) {
    for (object_id, object, props) in object_query.iter() {
        if let Some(friction) = props.get_float("friction") {
            println!("Object {}: friction = {}", object_id.0, friction);
        }
    }
}
```

---

## Summary: Key Architectural Decisions

### ✅ What Layer 2 DOES

1. **Spawns entity hierarchy:**
   - Map entities
   - Layer entities (all types)
   - Individual object entities
   - **NOT individual tile entities**

2. **Provides pre-processed data:**
   - `TileLayerData` with `TileInstance` grid
   - `TiledObject` with pre-computed vertices
   - `MergedProperties` with inheritance resolved
   - `ImageLayerData` with image handles
   - All tileset handles embedded where needed

3. **Establishes relationships:**
   - Bevy relationship system for bidirectional traversal
   - `TiledLayerMapOf` / `LayersInMap`
   - `TiledObjectMapOf` / `ObjectsInMap`
   - `TiledWorldOf` / `MapsInWorld`

4. **Emits events for Layer 3:**
   - `TileLayerCreated` (data ready for rendering)
   - `ObjectLayerCreated`, `ImageLayerCreated`
   - `ObjectCreated` (individual objects)

### ❌ What Layer 2 Does NOT Do

1. **No individual tile entities** - tiles are data in `TileLayerData`, Layer 3 decides rendering
2. **No TiledMapStorage** - redundant with relationships + component queries
3. **No rendering components** - Layer 3 adds Sprite, TilemapBundle, etc.
4. **No physics components** - Layer 3 adds Collider, RigidBody, etc.
5. **No SpawnContext in events** - only used internally, data in queryable components

### 🎯 Design Rationale

| Decision | Why |
|----------|-----|
| **Don't spawn tile entities** | Rendering optimizations require Layer 3 control (bevy_ecs_tilemap uses TileStorage, native tilemaps use components, not individual entities with Transforms) |
| **TileLayerData component** | Pre-processes GID → (tileset_handle, local_id, flip_flags). Layer 3 iterates once to build optimal rendering structure |
| **Pre-compute shape vertices** | Physics + debug rendering both need Vec<Vec2>. Compute once, store in component, avoid 2x recomputation |
| **Remove TiledMapStorage** | Relationships provide traversal. Component queries provide ID lookups (rare). TileInstance has tileset handles |
| **Relationship system** | Bidirectional queries, automatic sync, cleaner than manual HashMap management |
| **MergedProperties** | Compute inheritance once at spawn time. Layer 3 queries for properties without re-merging |

---

## Next Steps After Layer 2

Once Layer 2 is complete:
- **Layer 3a**: bevy_tiled_render - sprite/tilemap rendering
- **Layer 3b**: bevy_tiled_tilemap - bevy_ecs_tilemap integration
- **Layer 3c**: bevy_tiled_avian - Avian physics collider generation
- **Layer 3d**: Custom plugins - user-specific functionality
