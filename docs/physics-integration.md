# Physics Integration: Avian Backend

This document describes how physics integration works in `bevy_tiled_topdown`, focusing exclusively on the **Avian** physics engine (we're dropping Rapier support).

---

## Overview

The physics system generates colliders from:
1. **Tile layers** - Uses collision objects defined in tilesets
2. **Object layers** - Uses object geometry (rectangles, polygons, etc.)

The system is **event-driven** and **backend-abstracted**, allowing customization while maintaining a clean separation from core map loading.

---

## Backend Abstraction

### TiledPhysicsBackend Trait

All physics backends implement this trait:

```rust
pub trait TiledPhysicsBackend:
    Default + Clone + Debug + 'static + Sync + Send + FromReflect + Reflectable
{
    /// Spawn colliders for the given geometry
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: &TiledEvent<ColliderCreated>,
        multi_polygon: &geo::MultiPolygon<f32>,
    ) -> Vec<Entity>;
}
```

**Key design:**
- Backend receives **pre-processed geometry** (`geo::MultiPolygon<f32>`)
- Framework handles polygon extraction, merging, and simplification
- Backend only needs to convert geometry to physics colliders
- Returns `Vec<Entity>` of spawned collider entities

---

## Avian Backend Implementation

### Backend Strategies

The Avian backend offers **three collider generation strategies**:

```rust
#[derive(Default, Reflect, Copy, Clone, Debug)]
pub enum TiledPhysicsAvianBackend {
    #[default]
    Polyline,       // Single polyline collider with indexed vertices
    Triangulation,  // Compound collider with triangulated polygons
    LineStrip,      // Multiple linestrip colliders (one per line string)
}
```

### Strategy Details

#### 1. Polyline (Default - Recommended)

**Use case:** General-purpose static geometry, good balance of performance and accuracy

**How it works:**
- Extracts all line segments from polygon boundaries
- Creates indexed vertex/indices arrays
- Single `Collider` with `SharedShape::polyline(vertices, Some(indices))`

**Pros:**
- Fast collision detection
- Minimal entity overhead (one collider per object/layer)
- Memory efficient

**Cons:**
- Hollow (only boundaries, not solid fill)
- Not suitable for dynamic rigidbodies

**Example:**
```rust
TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
    backend: TiledPhysicsAvianBackend::Polyline,
    // ...
}
```

#### 2. Triangulation

**Use case:** Solid colliders for dynamic rigidbodies, complex shapes

**How it works:**
- Uses Delaunay triangulation to fill polygons
- Creates compound shape with multiple triangle colliders
- Each triangle positioned at its centroid with local vertices

**Pros:**
- Solid fill (suitable for dynamic bodies)
- Handles complex/concave shapes well

**Cons:**
- More expensive collision detection
- Higher memory usage (many triangles)

**Example:**
```rust
TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
    backend: TiledPhysicsAvianBackend::Triangulation,
    // ...
}
```

#### 3. LineStrip

**Use case:** Debugging, per-segment collision detection

**How it works:**
- Creates one collider per line string
- Each uses `SharedShape::polyline(vertices, None)`
- Multiple child entities

**Pros:**
- Fine-grained collision detection
- Easy to visualize/debug individual segments

**Cons:**
- Many entities (can be hundreds for complex shapes)
- Higher overhead

**Example:**
```rust
TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
    backend: TiledPhysicsAvianBackend::LineStrip,
    // ...
}
```

---

## Configuration

### TiledPhysicsSettings Component

Attach this component to `TiledMap` or `TiledWorld` entities to enable physics:

```rust
#[derive(Component, Reflect, Clone, Debug)]
pub struct TiledPhysicsSettings<T: TiledPhysicsBackend> {
    /// Which object layers to process
    pub objects_layer_filter: TiledFilter,

    /// Which objects to generate colliders for
    pub objects_filter: TiledFilter,

    /// Which tile layers to process
    pub tiles_layer_filter: TiledFilter,

    /// Which tile collision objects to use
    pub tiles_objects_filter: TiledFilter,

    /// Backend configuration
    pub backend: T,
}
```

### TiledFilter Options

```rust
pub enum TiledFilter {
    All,                      // Match everything
    Names(Vec<String>),       // Match specific names (case-insensitive)
    RegexSet(regex::RegexSet), // Match regex patterns
    None,                     // Match nothing
}
```

**Examples:**

```rust
// Only generate colliders from layers named "Collision"
tiles_layer_filter: TiledFilter::Names(vec!["Collision".into()])

// Generate colliders from all layers
tiles_layer_filter: TiledFilter::All

// Use regex to match layers starting with "col_"
tiles_layer_filter: TiledFilter::RegexSet(
    RegexSet::new(["^col_.*"]).unwrap()
)

// Don't generate colliders from any tile layers
tiles_layer_filter: TiledFilter::None
```

---

## Usage Examples

### Basic Setup

```rust
use bevy::prelude::*;
use bevy_tiled_topdown::prelude::*;
use avian2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TiledPlugin::default())
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        .add_plugins(PhysicsPlugins::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap {
            handle: asset_server.load("maps/level1.tmx"),
        },
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            backend: TiledPhysicsAvianBackend::Polyline,
            tiles_layer_filter: TiledFilter::Names(vec!["Collision".into()]),
            objects_layer_filter: TiledFilter::All,
            objects_filter: TiledFilter::All,
            tiles_objects_filter: TiledFilter::Names(vec!["collision".into()]),
        },
    ));
}
```

### Filtering Specific Objects

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap {
            handle: asset_server.load("maps/dungeon.tmx"),
        },
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            backend: TiledPhysicsAvianBackend::Polyline,

            // Only process "Collision" tile layer
            tiles_layer_filter: TiledFilter::Names(vec!["Collision".into()]),

            // Process all object layers
            objects_layer_filter: TiledFilter::All,

            // Only objects named "wall" or "door"
            objects_filter: TiledFilter::Names(vec!["wall".into(), "door".into()]),

            // All tile collision objects
            tiles_objects_filter: TiledFilter::All,
        },
    ));
}
```

### Using Triangulation for Dynamic Bodies

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap {
            handle: asset_server.load("maps/moving_platforms.tmx"),
        },
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            // Use triangulation for solid colliders
            backend: TiledPhysicsAvianBackend::Triangulation,

            tiles_layer_filter: TiledFilter::Names(vec!["Platforms".into()]),
            objects_layer_filter: TiledFilter::All,
            objects_filter: TiledFilter::All,
            tiles_objects_filter: TiledFilter::All,
        },
    ));
}
```

---

## Event-Driven Integration

### Observing Collider Creation

The physics system emits `TiledEvent<ColliderCreated>` events that you can observe:

```rust
fn setup_observers(app: &mut App) {
    app.add_observer(on_collider_created);
}

fn on_collider_created(
    trigger: On<TiledEvent<ColliderCreated>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let source = trigger.event.source;

    match source {
        TiledColliderSource::TilesLayer => {
            // Collider from tile layer - make it static
            commands.entity(entity).insert((
                RigidBody::Static,
                CollisionLayers::new([Layer::Ground], [Layer::Player]),
            ));
        }
        TiledColliderSource::Object => {
            // Collider from object - customize based on object properties
            commands.entity(entity).insert((
                RigidBody::Static,
                Sensor,
                CollisionLayers::new([Layer::Trigger], [Layer::Player]),
            ));
        }
    }
}
```

### Adding Custom Components

```rust
#[derive(Component, Reflect)]
pub struct Wall;

#[derive(Component, Reflect)]
pub struct Sensor {
    pub trigger_event: String,
}

fn on_object_created(
    trigger: On<TiledEvent<ObjectCreated>>,
    q_objects: Query<&Name>,
    mut commands: Commands,
) {
    if let Ok(name) = q_objects.get(trigger.entity) {
        if name.as_str() == "wall" {
            commands.entity(trigger.entity).insert(Wall);
        }
    }
}

fn on_collider_created(
    trigger: On<TiledEvent<ColliderCreated>>,
    q_walls: Query<&Wall>,
    mut commands: Commands,
) {
    // Check if parent object is a wall
    let parent = trigger.event.origin;
    if q_walls.contains(parent) {
        commands.entity(trigger.entity).insert((
            RigidBody::Static,
            CollisionLayers::new([Layer::Wall], [Layer::Player, Layer::Enemy]),
        ));
    }
}
```

---

## Geometry Processing

### From Tile Layers

1. **Extract collision objects** from each tile's collision data (defined in tileset)
2. **Transform geometry** based on tile position, flip_h, flip_v, rotation
3. **Accumulate polygons** from all tiles in the layer
4. **Merge adjacent polygons** using Boolean union operations (optimization)
5. **Pass to backend** as `geo::MultiPolygon<f32>`

**Note:** Colliders are spawned at the **tilemap level**, not per-tile (efficient for large maps).

### From Objects

1. **Extract object geometry** (Rectangle, Polygon, Polyline, Ellipse, etc.)
2. **For tile objects:** Inherit tile's collision data, apply transformations
3. **For shape objects:** Convert shape to polygon representation
4. **Pass to backend** as `geo::MultiPolygon<f32>`

**Note:** Each object gets its own collider(s).

### Optimization: Polygon Merging

The system uses a divide-and-conquer algorithm to merge adjacent polygons:

```rust
// Simplify geometry: merge together adjacent polygons
let polygons = divide_reduce(polygons, |a, b| a.union(&b));
```

**Benefits:**
- Fewer colliders (better performance)
- Cleaner collision boundaries
- Reduced memory usage

---

## Entity Hierarchy

Colliders are spawned as **children** of their source:

```
Map Entity (TiledMap)
├─ Layer Entity (TiledLayer::Tiles)
│  └─ Tilemap Entity (TiledTilemap)
│     └─ Collider Entity (TiledColliderOf)
│        ├─ Collider (Avian component)
│        ├─ TiledColliderSource::TilesLayer
│        ├─ TiledColliderPolygons (raw geometry)
│        └─ Transform
│
└─ Layer Entity (TiledLayer::Objects)
   └─ Object Entity (TiledObject)
      └─ Collider Entity (TiledColliderOf)
         ├─ Collider (Avian component)
         ├─ TiledColliderSource::Object
         ├─ TiledColliderPolygons (raw geometry)
         └─ Transform
```

**Relationship components:**
- `TiledColliderOf(parent_entity)` - On collider, points to parent
- `TiledColliders(Vec<Entity>)` - On parent, lists all child colliders

---

## Component Reference

### TiledColliderSource

```rust
#[derive(Component, Reflect, Copy, Clone, Debug)]
pub enum TiledColliderSource {
    TilesLayer,  // Collider from tile layer
    Object,      // Collider from object
}
```

**Usage:** Distinguish between tile-based and object-based colliders.

### TiledColliderOf

```rust
#[derive(Component, Reflect, Clone, Debug)]
pub struct TiledColliderOf(pub Entity);
```

**Usage:** Points to parent entity (TiledObject or TiledTilemap).

### TiledColliders

```rust
#[derive(Component, Reflect, Debug, Deref)]
pub struct TiledColliders(Vec<Entity>);
```

**Usage:** On parent entity, lists all child colliders.

### TiledColliderPolygons

```rust
#[derive(Component, Reflect, Clone, Debug)]
pub struct TiledColliderPolygons(pub geo::MultiPolygon<f32>);
```

**Usage:** Stores raw polygon geometry (useful for debugging/visualization).

---

## Performance Considerations

### Strategy Selection

| Strategy | Entities | Collision Speed | Memory | Best For |
|----------|----------|----------------|--------|----------|
| **Polyline** | Low (1 per source) | Fast | Low | Static geometry, boundaries |
| **Triangulation** | Low (1 per source) | Medium | Medium | Dynamic bodies, solid fill |
| **LineStrip** | High (many per source) | Slow | High | Debugging, fine-grained detection |

### Filtering Best Practices

1. **Be specific:** Only process layers/objects that need colliders
   ```rust
   tiles_layer_filter: TiledFilter::Names(vec!["Collision".into()])
   // Instead of TiledFilter::All
   ```

2. **Use regex for patterns:** Match multiple similar names efficiently
   ```rust
   tiles_layer_filter: TiledFilter::RegexSet(
       RegexSet::new(["^collision_.*", "^wall_.*"]).unwrap()
   )
   ```

3. **Separate collision layers:** In Tiled, put collision-only tiles in dedicated layers
   - Makes filtering easier
   - Cleaner map organization
   - Better performance

### Large Maps

For large maps:
1. Use **TiledWorld** with **TiledWorldChunking** (load maps on-demand)
2. Use **Polyline** backend (fewest entities)
3. Filter aggressively (only necessary collision layers)
4. Consider spatial partitioning for collision detection (Avian handles this)

---

## Migration from Rapier

If you were using `bevy_ecs_tiled` with Rapier:

### What's the Same

- `TiledPhysicsBackend` trait (identical API)
- `TiledPhysicsSettings<T>` component (same structure)
- `TiledFilter` options (unchanged)
- Event-driven architecture (same patterns)
- Geometry processing (same algorithm)

### What's Different

1. **Backend enum:**
   ```rust
   // Old (Rapier)
   TiledPhysicsRapierBackend::Polyline

   // New (Avian)
   TiledPhysicsAvianBackend::Polyline
   ```

2. **Plugin:**
   ```rust
   // Old
   .add_plugins(TiledPhysicsPlugin::<TiledPhysicsRapierBackend>::default())

   // New
   .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
   ```

3. **Collider components:** Avian uses different component names than Rapier, but the physics settings component is the same

4. **Dependencies:**
   ```toml
   # Old
   bevy_rapier2d = "0.27"

   # New
   avian2d = "0.4"
   ```

### Migration Steps

1. Replace `rapier` with `avian` in dependencies
2. Update type parameters: `TiledPhysicsRapierBackend` → `TiledPhysicsAvianBackend`
3. Update plugin: `TiledPhysicsPlugin::<TiledPhysicsRapierBackend>` → `TiledPhysicsPlugin::<TiledPhysicsAvianBackend>`
4. Update observer code if you reference Rapier-specific components
5. Test collision behavior (Avian may have different collision resolution)

---

## Troubleshooting

### No colliders spawning

**Check:**
1. Is `TiledPhysicsPlugin::<TiledPhysicsAvianBackend>` added?
2. Is `TiledPhysicsSettings` component on the map entity?
3. Are filters too restrictive? (Try `TiledFilter::All` for debugging)
4. Do tiles have collision objects defined in the tileset?
5. Check console for warnings/errors

### Colliders in wrong position

**Possible causes:**
1. Map transform not identity (colliders are in local space)
2. Tile flip/rotation not accounted for (should be automatic)
3. Coordinate system mismatch (Tiled Y-down vs Bevy Y-up)

**Solution:** Ensure map entity has `Transform::IDENTITY`, the framework handles coordinate conversion.

### Too many colliders

**Solutions:**
1. Use `Polyline` instead of `LineStrip`
2. Enable polygon merging (automatic in tile layers)
3. Filter more aggressively
4. Combine collision tiles in Tiled

### Performance issues

**Solutions:**
1. Use `Polyline` backend (fastest)
2. Filter to only necessary layers/objects
3. Use `TiledWorldChunking` for large worlds
4. Simplify collision geometry in Tiled
5. Disable debug rendering in production

---

## Advanced Usage

### Custom Backend

You can implement your own backend:

```rust
#[derive(Default, Reflect, Clone, Debug)]
pub struct MyCustomBackend {
    pub my_setting: f32,
}

impl TiledPhysicsBackend for MyCustomBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: &TiledEvent<ColliderCreated>,
        multi_polygon: &geo::MultiPolygon<f32>,
    ) -> Vec<Entity> {
        // Your custom collider spawning logic
        vec![]
    }
}

// Use it:
.add_plugins(TiledPhysicsPlugin::<MyCustomBackend>::default())
```

### Dynamic Collider Modification

```rust
fn modify_colliders(
    trigger: On<TiledEvent<ColliderCreated>>,
    mut commands: Commands,
) {
    // Modify collider right after creation
    commands.entity(trigger.entity).insert(/* your components */);
}
```

### Collision Groups

```rust
use avian2d::prelude::*;

fn on_collider_created(
    trigger: On<TiledEvent<ColliderCreated>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;

    // Set collision groups based on source
    match trigger.event.source {
        TiledColliderSource::TilesLayer => {
            commands.entity(entity).insert(
                CollisionLayers::new([Layer::Ground], [Layer::Player, Layer::Enemy])
            );
        }
        TiledColliderSource::Object => {
            commands.entity(entity).insert(
                CollisionLayers::new([Layer::Wall], [Layer::Player])
            );
        }
    }
}
```

---

## Summary

- **Avian only** - Rapier support dropped
- **Three strategies** - Polyline (default), Triangulation, LineStrip
- **Event-driven** - Observe `ColliderCreated` to customize
- **Filtered** - Use `TiledFilter` to control what gets colliders
- **Pre-processed** - Backend receives clean `geo::MultiPolygon<f32>`
- **Optimized** - Polygon merging reduces collider count
- **Hierarchical** - Colliders are children of their source entities
