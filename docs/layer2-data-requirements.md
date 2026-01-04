# Layer 2 Data Requirements Analysis

This document maps out **concrete data requirements** for Layer 3 plugins (rendering & physics), then designs how Layer 2 (bevy_tiled_core) will provide this data ergonomically.

## Approach

Instead of designing abstractions in a vacuum, we start with two concrete Layer 3 use cases:
1. **Rendering** (bevy_tiled_render / bevy_tiled_tilemap)
2. **Physics** (bevy_tiled_avian)

Once we know exactly what data they need, we can justify Layer 2's architecture.

---

## Use Case 1: Rendering Plugin Data Requirements

### A. Tile Layer Rendering → bevy_ecs_tilemap Integration

| Data Needed | Source | When Needed | How Accessed |
|-------------|--------|-------------|--------------|
| **Tile GID** | Map layer tile data | Spawning each tile | Iterate map.layers()[i].tiles() |
| **Local Tile ID** | Derived from GID - first_gid | Spawning | `gid - tileset.first_gid` |
| **Tileset Reference** | Map's tileset list | Spawning | Lookup by first_gid range |
| **Texture Atlas** | Tileset asset | Rendering system | `tileset_asset.atlas_image` |
| **Tile Size** | Tileset | Spawning/Rendering | `tileset.tile_size` |
| **Grid Dimensions** | Tileset | Rendering | `tileset.grid_size` (columns × rows) |
| **Spacing/Margin** | Tileset | Rendering | `tileset.spacing`, `tileset.margin` |
| **Flip Flags** | Encoded in GID | Spawning | Extract from GID bits (H/V/D flip) |
| **Tile Animation** | Tileset tile definition | Rendering system | `tileset.tiles()[local_id].animation` |
| **Layer Opacity** | Layer properties | Rendering | `layer.opacity` |
| **Layer Tint** | Layer properties | Rendering | `layer.tint_color` |
| **Layer Parallax** | Layer properties | Camera system | `layer.parallax_factor` |
| **Layer Offset** | Layer properties | Transform | `layer.offset` |
| **Z-Order** | Layer index | Transform.z | Derived from layer order |

**Access Pattern:**
```rust
fn spawn_tile_layer(
    commands: &mut Commands,
    layer: &TileLayer,
    context: &SpawnContext,  // ← Needs this!
) {
    for (x, y, tile_gid) in layer.tiles() {
        // Need: GID → Tileset lookup
        let (tileset_asset, local_id) = context.resolve_gid(tile_gid)?;

        // Need: Flip flags from GID
        let (flipped_h, flipped_v, flipped_d) = context.extract_flip_flags(tile_gid);

        // Need: Animation data from tileset
        let animation = tileset_asset.tileset.tiles()
            .get(local_id)
            .and_then(|t| t.animation.clone());

        commands.spawn((
            TiledTile {
                tile_id: local_id,
                tileset_index: /* ??? */,
                flipped_h,
                flipped_v,
                flipped_d,
            },
            TileAnimation(animation),  // Pre-computed, stored in component
            Transform::from_xyz(x * tile_width, y * tile_height, layer_z),
        ));
    }
}
```

### B. Object Layer Rendering → Tile Objects

| Data Needed | Source | When Needed | How Accessed |
|-------------|--------|-------------|--------------|
| **Object Type** | Object.shape | Spawning | Match on ObjectShape enum |
| **Position** | Object x, y | Transform | Direct read |
| **Rotation** | Object.rotation | Transform | Direct read |
| **Size** | Object width, height | Spawning | Direct read |
| **Tile GID** | Object.tile.gid (if tile object) | Spawning | `object.tile.map(|t| t.gid)` |
| **Tileset Lookup** | GID → Tileset | Spawning | Same as tile layers |
| **Template Properties** | Template asset | Spawning | Merge with object properties |
| **Object Properties** | Object.properties | Spawning | Direct read |

**Access Pattern:**
```rust
fn spawn_object_layer(
    commands: &mut Commands,
    layer: &ObjectLayer,
    context: &SpawnContext,  // ← Needs this!
) {
    for object in layer.objects() {
        let shape = match object.shape {
            ObjectShape::Tile { gid, width, height } => {
                // Need: GID → Tileset lookup (same as tiles)
                let (tileset_asset, local_id) = context.resolve_gid(gid)?;

                TiledObject::Tile {
                    tile_id: local_id,
                    tileset_index: /* ??? */,
                    width,
                    height,
                }
            }
            ObjectShape::Polygon { points } => {
                // Need: Pre-compute vertices ONCE
                let vertices: Vec<Vec2> = points.iter()
                    .map(|(x, y)| Vec2::new(*x, *y))
                    .collect();

                TiledObject::Polygon { vertices }  // Stored in component
            }
            // ... other shapes
        };

        // Need: Merged properties (object + template)
        let merged_props = context.get_merged_properties(object.id())?;

        commands.spawn((
            shape,
            Transform::from_xyz(object.x, object.y, layer_z),
            MergedProperties(merged_props),  // Pre-computed
        ));
    }
}
```

---

## Use Case 2: Physics Plugin Data Requirements

### A. Object Collision Shapes → Direct Shapes

| Data Needed | Source | When Needed | How Accessed |
|-------------|--------|-------------|--------------|
| **Shape Type** | Object.shape | Spawning collider | Match on ObjectShape |
| **Rectangle Dimensions** | width, height | Box collider | Direct from object |
| **Ellipse Dimensions** | width, height | Circle/Capsule | Direct from object |
| **Polygon Vertices** | points array | Polygon collider | Convert to Vec<Vec2> |
| **Polyline Vertices** | points array | Edge collider | Convert to Vec<Vec2> |
| **Position** | x, y | Transform (already set) | Read from Transform component |
| **Rotation** | rotation | Transform (already set) | Read from Transform component |
| **Physics Properties** | Custom properties | Collider material | friction, restitution, etc. |

**Access Pattern:**
```rust
fn add_colliders_to_objects(
    trigger: On<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    object_query: Query<(&TiledObject, &MergedProperties)>,
) {
    let (object, props) = object_query.get(trigger.entity).unwrap();

    // Shape data ALREADY computed in TiledObject component
    let collider = match object {
        TiledObject::Rectangle { width, height } => {
            Collider::rectangle(*width, *height)
        }
        TiledObject::Polygon { vertices } => {
            Collider::convex_hull(vertices.clone())  // Already Vec<Vec2>
        }
        TiledObject::Ellipse { width, height } => {
            Collider::circle(width.min(height) / 2.0)
        }
        _ => return,
    };

    // Physics properties from custom properties
    let friction = props.get_float("friction").unwrap_or(0.5);
    let restitution = props.get_float("restitution").unwrap_or(0.0);
    let is_sensor = props.get_bool("is_sensor").unwrap_or(false);

    commands.entity(trigger.entity).insert((
        collider,
        Friction::new(friction),
        Restitution::new(restitution),
        Sensor(is_sensor),
    ));
}
```

### B. Tile Collision Shapes → From Tileset

| Data Needed | Source | When Needed | How Accessed |
|-------------|--------|-------------|--------------|
| **Tile GID** | Already in TiledTile | Collision spawn | Read from component |
| **Tileset Asset** | Need handle | Collision spawn | ??? How to get from tile entity? |
| **Tile Collision Object** | tileset.tiles()[id].collision | Collision spawn | Nested ObjectLayer in tile def |
| **Collision Shapes** | tile.collision.objects | Collision spawn | Iterate collision objects |
| **Shape Vertices** | object.points | Collider | Same as object shapes |

**Access Pattern (THIS IS THE PROBLEM):**
```rust
fn add_colliders_to_tiles(
    trigger: On<TiledEvent<TileCreated>>,
    mut commands: Commands,
    tile_query: Query<&TiledTile>,
    // ??? How to get tileset asset from tile entity?
    tileset_assets: Res<Assets<TiledTilesetAsset>>,  // Have this
    // ??? But need the Handle<TiledTilesetAsset> for this tile
) {
    let tile = tile_query.get(trigger.entity).unwrap();

    // PROBLEM: How do I get the tileset handle from tile.tileset_index?
    // Need: tile.tileset_index → Handle<TiledTilesetAsset>
    let tileset_handle = ???;  // ← MISSING LINK

    let tileset = tileset_assets.get(&tileset_handle).unwrap();
    let tile_def = tileset.tileset.tiles().get(tile.tile_id)?;

    if let Some(collision) = &tile_def.collision {
        for obj in collision.objects() {
            // Spawn collider from obj.shape
        }
    }
}
```

**Key Problem:** Tile entity needs a way to reference its tileset asset handle!

---

## Data Flow Analysis

### Current Layer 1 → Layer 2 → Layer 3 Flow

```
Layer 1 (Assets):
  TiledMapAsset {
    map: Map,                                    // Raw tiled::Map
    tilesets: HashMap<u32, TilesetReference>,    // first_gid → Handle
    properties: Properties,
    layer_properties: HashMap<u32, Properties>,
    object_properties: HashMap<u32, Properties>,
  }

  TiledTilesetAsset {
    tileset: Tileset,                            // Raw tiled::Tileset
    atlas_image: Option<Handle<Image>>,
    tile_images: HashMap<u32, Handle<Image>>,
    properties: Properties,
    tile_properties: HashMap<u32, Properties>,
  }

Layer 2 (Spawning):
  ??? What does it provide to Layer 3?

Layer 3 (Rendering/Physics):
  Needs:
  1. GID → Tileset lookup
  2. Tileset handle from tile entity
  3. Pre-computed shape data
  4. Merged properties
  5. Asset data access during spawning
```

---

## Critical Questions to Answer

### Q1: How does a tile entity reference its tileset? (option a)

**Option A: Store tileset handle in component** (this one!)
```rust
#[derive(Component)]
pub struct TiledTile {
    pub tile_id: u32,
    pub tileset_handle: Handle<TiledTilesetAsset>,  // ← Direct reference
    pub flipped_h: bool,
    pub flipped_v: bool,
    pub flipped_d: bool,
}
```
**Pros:** Layer 3 can directly access tileset asset
**Cons:** Duplicates handle across all tiles from same tileset

**Option B: Store tileset index, provide lookup**
```rust
#[derive(Component)]
pub struct TiledTile {
    pub tile_id: u32,
    pub tileset_index: u32,  // ← Index into... what?
    // ...
}

// Somewhere else:
pub struct TiledMapStorage {
    pub tilesets: Vec<Handle<TiledTilesetAsset>>,  // Index lookup
}
```
**Pros:** Less duplication
**Cons:** Requires TiledMapStorage lookup

**Option C: Use relationship to TiledTilemap entity**
```rust
// Tile entity has:
TiledLayerOf(layer_entity)

// Layer entity has child TiledTilemap entities per tileset:
TiledTilemap {
    tileset_handle: Handle<TiledTilesetAsset>,
    tileset_index: u32,
}

// Tiles belong to TiledTilemap, not directly to layer
TiledTilemapOf(tilemap_entity)
```
**Pros:** Natural hierarchy (matches bevy_ecs_tilemap pattern)
**Cons:** Requires traversing relationship to get tileset

### Q2: When/how do we compute shape data? (option a)

**Option A: Compute during spawning, store in component** (this one!)
```rust
// In spawn_objects_layer():
let vertices: Vec<Vec2> = object.points.iter()
    .map(|(x, y)| Vec2::new(*x, *y))
    .collect();

commands.spawn(TiledObject::Polygon { vertices });  // Pre-computed
```
**Pros:** Only computed once, Layer 3 just reads
**Cons:** Component size larger, but probably fine

**Option B: Store raw data, compute in Layer 3**
```rust
// In spawn_objects_layer():
commands.spawn(TiledObject::Polygon {
    points: object.points.clone(),  // Raw f64 tuples
});

// In Layer 3:
let vertices: Vec<Vec2> = points.iter().map(...).collect();  // Compute again
```
**Pros:** Smaller components
**Cons:** Layer 3 recomputes for every use (physics + debug rendering = 2x)

### Q3: How does Layer 3 access asset data during spawning? (option b)

**Option A: Pass SpawnContext in events**
```rust
#[derive(Event)]
pub struct TiledEvent<E> {
    pub entity: Entity,
    pub event: E,
    pub context: SpawnContext,  // ← Contains asset references
}

// Layer 3:
fn handle_tile_created(
    trigger: On<TiledEvent<TileCreated>>,
) {
    let tileset = trigger.context.get_tileset(tile.tileset_index)?;
    // Use tileset data
}
```
**Pros:** Ergonomic, everything available
**Cons:** SpawnContext lifetime/cloning issues

**Option B: Layer 3 queries Assets directly** (probably this one for now, maybe a trait or system param something could make this nicer eventually)
```rust
fn handle_tile_created(
    trigger: On<TiledEvent<TileCreated>>,
    tile_query: Query<&TiledTile>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
) {
    let tile = tile_query.get(trigger.entity).unwrap();
    // Still need: tile → tileset handle lookup
    let tileset = tileset_assets.get(&???).unwrap();
}
```
**Pros:** No lifetime issues
**Cons:** Every Layer 3 plugin needs asset queries + handle lookup

**Option C: Store necessary data in components, skip asset access**
```rust
// Pre-compute everything Layer 3 might need:
#[derive(Component)]
pub struct TileCollisionData {
    pub shapes: Vec<CollisionShape>,  // Pre-extracted from tileset
}

// Layer 3:
fn handle_tile_created(
    trigger: On<TiledEvent<TileCreated>>,
    collision_query: Query<&TileCollisionData>,
) {
    if let Ok(collision) = collision_query.get(trigger.entity) {
        // Just use pre-computed data
    }
}
```
**Pros:** Layer 3 never touches assets
**Cons:** Layer 2 must anticipate all needs, bloated components

### Q4: Do we need TiledMapStorage for ID lookups? (option b)

**Scenario: User wants to find entity for Tiled object ID 42**

**Option A: Keep TiledMapStorage**
```rust
#[derive(Component)]
pub struct TiledMapStorage {
    pub layers: HashMap<u32, Entity>,        // Tiled layer ID → Entity
    pub objects: HashMap<u32, Entity>,       // Tiled object ID → Entity
    pub tilesets: Vec<Handle<TiledTilesetAsset>>,  // Index lookup
}

// Usage:
fn find_door_object(
    map_query: Query<&TiledMapStorage, With<TiledMap>>,
) {
    let storage = map_query.single();
    let door_entity = storage.objects.get(&42)?;
}
```
**Pros:** O(1) lookup
**Cons:** Duplicate data (entity already has ObjectId component), manual sync

**Option B: Query for ObjectId** (this one)
```rust
#[derive(Component)]
pub struct ObjectId(pub u32);  // Tiled's object ID

// Usage:
fn find_door_object(
    object_query: Query<(Entity, &ObjectId)>,
) {
    let door_entity = object_query.iter()
        .find(|(_, id)| id.0 == 42)
        .map(|(e, _)| e)?;
}
```
**Pros:** No duplicate data, ECS queries are fast
**Cons:** O(n) linear search (but n is usually small per map)

**Usage Frequency Analysis:**
- Rendering: Never needs ID lookup (uses Added<TiledTile> queries)
- Physics: Never needs ID lookup (uses events/queries)
- Game Logic: Rare (maybe "find spawn point by name" once at level load)

**Verdict:** ID lookups are rare enough that O(n) query is acceptable. Don't need HashMap.

---

## Proposed Layer 2 Data Provisioning Design

Based on the above analysis:

### CRITICAL DECISION: Layer 2 Does NOT Spawn Tile Entities

**Rationale:**
- Spawning individual entities for tiles (with Transform, etc.) defeats tilemap rendering optimizations
- Layer 3 rendering plugins need freedom to optimize (bevy_ecs_tilemap, Bevy native tilemaps, etc.)
- Different rendering approaches have different entity structures:
  - bevy_ecs_tilemap: TileStorage with batched rendering
  - Bevy native: Potential tilemap component
  - Simple sprites: Individual entities (user choice)

**Layer 2 provides RAW TILE DATA, Layer 3 decides how to render it.**

### 1. Component Design

```rust
// ===== TILE LAYERS (Data Only, No Individual Tile Entities) =====

#[derive(Component)]
pub struct TileLayerData {
    pub width: u32,                              // Map width in tiles
    pub height: u32,                             // Map height in tiles
    pub tiles: Vec<Option<TileInstance>>,        // Flattened grid [y * width + x]
}

#[derive(Clone, Debug)]
pub struct TileInstance {
    pub gid: u32,                                // Original GID (for reference)
    pub tileset_handle: Handle<TiledTilesetAsset>,  // Which tileset
    pub tile_id: u32,                            // Local ID in tileset
    pub flipped_h: bool,
    pub flipped_v: bool,
    pub flipped_d: bool,
}

impl TileLayerData {
    /// Get tile at position (returns None if out of bounds or empty)
    pub fn get(&self, x: u32, y: u32) -> Option<&TileInstance> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.get((y * self.width + x) as usize)?.as_ref()
    }

    /// Iterate all tiles with positions
    pub fn iter_tiles(&self) -> impl Iterator<Item = (u32, u32, &TileInstance)> {
        self.tiles.iter().enumerate().filter_map(|(idx, tile)| {
            tile.as_ref().map(|t| {
                let x = (idx as u32) % self.width;
                let y = (idx as u32) / self.width;
                (x, y, t)
            })
        })
    }
}

// ===== OBJECTS =====

#[derive(Component)]
pub enum TiledObject {
    Point,
    Rectangle { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polygon { vertices: Vec<Vec2> },      // Pre-computed, ready to use
    Polyline { vertices: Vec<Vec2> },     // Pre-computed
    Tile {
        tile_id: u32,
        tileset_handle: Handle<TiledTilesetAsset>,  // Direct reference
        width: f32,
        height: f32,
    },
    Text { /* ... */ },
}

#[derive(Component)]
pub struct ObjectId(pub u32);  // Tiled's original ID

#[derive(Component)]
pub struct LayerId(pub u32);   // Tiled's original ID

// ===== PROPERTIES =====

#[derive(Component)]
pub struct MergedProperties {
    properties: Properties,  // Pre-merged (object + template)
}

impl MergedProperties {
    pub fn get_float(&self, key: &str) -> Option<f32> { /* ... */ }
    pub fn get_bool(&self, key: &str) -> Option<bool> { /* ... */ }
    // ... accessor methods
}

// ===== RELATIONSHIPS =====

#[derive(Component)]
#[relationship(relationship_target = LayersInMap)]
pub struct TiledLayerMapOf(pub Entity);

#[derive(Component)]
#[relationship(relationship_target = ObjectsInMap)]
pub struct TiledObjectMapOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TiledMapOf)]
pub struct LayersInMap(Vec<Entity>);

// For tile objects that reference tiles:
#[derive(Component)]
#[relationship(relationship_target = ObjectsUsingTile)]
pub struct TileObjectOf(pub Entity);  // Points to tile entity
```

### 2. Remove TiledMapStorage

**Justification:**
1. **Tileset handles**: Stored directly in TiledTile/TiledObject components
2. **Layer lookups**: Use LayersInMap relationship
3. **Object ID lookups**: Query for ObjectId component (rare operation)
4. **Tile lookups**: Use TiledTilemap's TileStorage (position-based)

**What was TiledMapStorage supposed to do?**
- ❌ Store tileset handles → Now in components directly
- ❌ Store layer entities → Now in LayersInMap relationship
- ❌ Store object ID → Entity map → Query ObjectId component instead

**Result:** TiledMapStorage is redundant. Remove it.

### 3. SpawnContext for Asset Access

During spawning, Layer 2 provides read-only asset access:

```rust
pub struct SpawnContext<'a> {
    pub map_asset: &'a TiledMapAsset,
    pub tileset_assets: &'a Assets<TiledTilesetAsset>,

    // Cached GID lookup
    tileset_ranges: Vec<(Range<u32>, Handle<TiledTilesetAsset>)>,
}

impl<'a> SpawnContext<'a> {
    /// Resolve GID to (tileset_handle, local_tile_id)
    pub fn resolve_gid(&self, gid: u32) -> Option<(Handle<TiledTilesetAsset>, u32)> {
        self.tileset_ranges.iter()
            .find(|(range, _)| range.contains(&gid))
            .map(|(range, handle)| (handle.clone(), gid - range.start))
    }

    /// Get merged properties for object (object + template)
    pub fn get_merged_properties(&self, object_id: u32) -> Option<Properties> {
        let obj_props = self.map_asset.object_properties.get(&object_id)?;
        // Merge with template if exists...
        Some(obj_props.clone())
    }
}
```

**Used during spawning ONLY, not passed to events.**

### 4. Event System (Minimal Context)

```rust
#[derive(Event)]
pub struct TiledEvent<E> {
    pub entity: Entity,      // The spawned entity
    pub map_entity: Entity,  // Parent map entity
    pub event: E,
}

// Specific events (marker types):
pub struct MapCreated;
pub struct LayerCreated;      // Generic layer created
pub struct TileLayerCreated;  // Specific: tile layer ready for rendering
pub struct ObjectLayerCreated;
pub struct ImageLayerCreated;
pub struct GroupLayerCreated;
pub struct ObjectCreated;     // Individual object spawned
// NO TileCreated - tiles aren't individual entities
```

**Layer 3 accesses data via components, not via event context:**

```rust
// Rendering plugin for tile layers:
fn spawn_tilemap_rendering(
    trigger: On<TiledEvent<TileLayerCreated>>,
    layer_query: Query<&TileLayerData>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let tile_data = layer_query.get(trigger.entity).unwrap();

    // Layer 3 decides how to render (example: bevy_ecs_tilemap)
    let mut tile_storage = TileStorage::empty(
        TilemapSize::new(tile_data.width, tile_data.height)
    );

    for (x, y, tile) in tile_data.iter_tiles() {
        let tileset = tileset_assets.get(&tile.tileset_handle).unwrap();

        let tile_entity = commands.spawn(TileBundle {
            position: TilePos::new(x, y),
            texture_index: TileTextureIndex(tile.tile_id),
            tilemap_id: TilemapId(trigger.entity),
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
        tile_size: TileSize::new(tileset.tile_size.x as f32, tileset.tile_size.y as f32),
        // ...
    });
}

// Physics plugin for objects:
fn add_colliders_to_objects(
    trigger: On<TiledEvent<ObjectCreated>>,
    object_query: Query<(&TiledObject, &MergedProperties)>,
    mut commands: Commands,
) {
    let (object, props) = object_query.get(trigger.entity).unwrap();

    // All data needed is in components!
    match object {
        TiledObject::Polygon { vertices } => {
            commands.entity(trigger.entity).insert(
                Collider::convex_hull(vertices.clone())
            );
        }
        _ => {}
    }
}
```

### 5. Hierarchy Design (REVISED)

```
TiledMap Entity
├─ TiledMap component (has Handle<TiledMapAsset>)
├─ LayersInMap relationship target (auto-synced list of children)
└─ Children:
   ├─ TiledLayer Entity (Tile Layer)
   │  ├─ TiledLayerMapOf(map_entity)
   │  ├─ TiledLayer::Tiles
   │  ├─ LayerId(u32)
   │  ├─ TileLayerData { width, height, tiles: Vec<TileInstance> }
   │  ├─ Transform (layer offset, parallax, etc.)
   │  └─ MergedProperties (layer properties)
   │  └─ Children: NONE (no individual tile entities!)
   │     └─ Layer 3 rendering may spawn tilemap entities here
   │
   ├─ TiledLayer Entity (Object Layer)
   │  ├─ TiledObjectMapOf(map_entity)
   │  ├─ TiledLayer::Objects
   │  ├─ LayerId(u32)
   │  ├─ Transform
   │  ├─ MergedProperties (layer properties)
   │  └─ Children:
   │     ├─ TiledObject Entity 1
   │     │  ├─ TiledObject::Polygon { vertices }
   │     │  ├─ ObjectId(42)
   │     │  ├─ MergedProperties (object + template)
   │     │  ├─ Transform
   │     │  └─ TiledObjectMapOf(map_entity)
   │     └─ TiledObject Entity 2...
   │
   ├─ TiledLayer Entity (Image Layer)
   │  ├─ TiledLayerMapOf(map_entity)
   │  ├─ TiledLayer::Image
   │  ├─ LayerId(u32)
   │  ├─ ImageLayerData { image_handle, width, height }
   │  ├─ Transform (position, opacity)
   │  └─ MergedProperties (layer properties)
   │
   └─ TiledLayer Entity (Group Layer)
      ├─ TiledLayerMapOf(map_entity)
      ├─ TiledLayer::Group
      ├─ LayerId(u32)
      ├─ Transform
      └─ Children: (recursive TiledLayer entities)
```

**Key Points:**
- **Tile layers:** Only ONE entity (the layer itself), with `TileLayerData` component
- **Object layers:** Layer entity + individual object entities (different shapes/behaviors)
- **Image layers:** Only ONE entity with image data
- **Group layers:** Hierarchical layer containers
- Both `TiledLayerMapOf`/`TiledObjectMapOf` (child → parent) AND `LayersInMap`/`ObjectsInMap` (parent → children)
- Objects have pre-computed vertices
- Properties pre-merged at layer AND object level

---

## Answers to Critical Questions

### Q1: How does a tile entity reference its tileset?
**Answer:** Store `tileset_handle: Handle<TiledTilesetAsset>` directly in `TiledTile` component.

**Justification:**
- Layer 3 physics needs tileset to get collision shapes
- Layer 3 rendering needs tileset for texture atlas
- Handle is cheap to clone (Arc internally)
- Duplication is acceptable (same as storing Transform on every entity)

### Q2: When/how do we compute shape data?
**Answer:** Compute during spawning, store in component.

**Justification:**
- Physics needs Vec<Vec2>, debug rendering needs Vec<Vec2>
- Computing twice (physics + debug) wastes CPU
- Pre-compute once in Layer 2, store in `TiledObject::Polygon { vertices: Vec<Vec2> }`
- Layer 3 just reads, no recomputation

### Q3: How does Layer 3 access asset data during spawning?
**Answer:** Via components. Layer 2 pre-computes and stores what's needed.

**Justification:**
- Layer 3 gets tileset via `TiledTile.tileset_handle`
- Layer 3 gets shapes via `TiledObject::Polygon { vertices }`
- Layer 3 gets properties via `MergedProperties`
- No asset queries needed in Layer 3 (except to load textures)

### Q4: Do we need TiledMapStorage for ID lookups?
**Answer:** No. Use relationships + component queries.

**Justification:**
- **Tileset lookups:** Stored in components directly (see Q1)
- **Layer lookups:** Use `LayersInMap` relationship (query.iter_descendants(map_entity))
- **Object ID lookups:** Query `(Entity, &ObjectId)` and filter (rare operation, O(n) acceptable)
- **Tile position lookups:** Use bevy_ecs_tilemap's TileStorage (position-based, not our concern)

**Usage frequency:**
- Rendering: 0 ID lookups (uses `Added<TiledTile>` queries)
- Physics: 0 ID lookups (uses events)
- Game logic: ~1-5 lookups at level load ("find player spawn point")

**Benchmarking:**
- Linear search through 1000 objects: ~1-10µs (negligible)
- HashMap overhead: Memory + sync complexity
- **Verdict:** Not worth the complexity. Remove TiledMapStorage.

---

## Final Layer 2 Architecture

### What Layer 2 Provides to Layer 3

1. **Pre-Computed Data Components:**
   - `TileLayerData` with grid of `TileInstance` (includes tileset handles, flip flags)
   - `TiledObject` with pre-computed vertices
   - `MergedProperties` with inheritance resolved
   - `ImageLayerData` with image handle + dimensions
   - `LayerId` and `ObjectId` for tracking

2. **Relationship System:**
   - `TiledLayerMapOf` / `LayersInMap` bidirectional traversal
   - `TiledObjectMapOf` / `ObjectsInMap` for object → map
   - `TiledWorldOf` / `MapsInWorld` for world support
   - Parent/Children for layer hierarchy (group layers)

3. **Event Hooks:**
   - `TiledEvent<TileLayerCreated>` when tile layer data ready
   - `TiledEvent<ObjectLayerCreated>` when object layer ready
   - `TiledEvent<ObjectCreated>` when individual object spawned
   - `TiledEvent<ImageLayerCreated>` when image layer ready
   - Observers for map/world creation

4. **Query-Friendly Design:**
   - `ObjectId(u32)` component for rare ID lookups
   - `LayerId(u32)` component for layer identification
   - Layer entities have `Transform` + `Visibility`
   - Object entities have `Transform` + `Visibility`
   - **NO individual tile entities** (Layer 3 decides rendering structure)

### What Layer 2 Does NOT Provide

- ❌ Individual tile entities (only TileLayerData component)
- ❌ TiledMapStorage (redundant with components + relationships)
- ❌ SpawnContext in events (only used internally during spawning)
- ❌ Rendering components (Sprite, TextureAtlas, TilemapBundle, etc.)
- ❌ Physics components (Collider, RigidBody, etc.)
- ❌ TileCreated events (tiles aren't entities)

### Layer 3 Usage Pattern

```rust
// Rendering plugin (bevy_ecs_tilemap integration):
fn spawn_tilemap_rendering(
    trigger: On<TiledEvent<TileLayerCreated>>,
    layer_query: Query<&TileLayerData>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let tile_data = layer_query.get(trigger.entity).unwrap();

    // Layer 3 has full control over rendering structure
    // Example: Using bevy_ecs_tilemap for batched rendering
    let mut tile_storage = TileStorage::empty(
        TilemapSize::new(tile_data.width, tile_data.height)
    );

    for (x, y, tile) in tile_data.iter_tiles() {
        let tileset = tileset_assets.get(&tile.tileset_handle).unwrap();

        let tile_entity = commands.spawn(TileBundle {
            position: TilePos::new(x, y),
            texture_index: TileTextureIndex(tile.tile_id),
            flip: TileFlip { x: tile.flipped_h, y: tile.flipped_v, d: tile.flipped_d },
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

// Physics plugin (objects):
fn add_colliders(
    trigger: On<TiledEvent<ObjectCreated>>,
    object_query: Query<(&TiledObject, &MergedProperties)>,
    mut commands: Commands,
) {
    let (object, props) = object_query.get(trigger.entity).unwrap();

    // Everything needed is in components:
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

// Physics plugin (tile layer collision):
fn add_tile_collision(
    trigger: On<TiledEvent<TileLayerCreated>>,
    layer_query: Query<(&TileLayerData, &MergedProperties)>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let (tile_data, layer_props) = layer_query.get(trigger.entity).unwrap();

    // Check if this layer should have collision
    if !layer_props.get_bool("has_collision").unwrap_or(false) {
        return;
    }

    // Iterate tiles and check for collision shapes in tileset definitions
    for (x, y, tile) in tile_data.iter_tiles() {
        let tileset = tileset_assets.get(&tile.tileset_handle).unwrap();

        if let Some(tile_def) = tileset.tileset.tiles().get(tile.tile_id) {
            if let Some(collision) = &tile_def.collision {
                // Spawn collider entities based on tileset collision shapes
                for collision_obj in collision.objects() {
                    // Spawn child collider entity
                }
            }
        }
    }
}
```

---

## Summary: Design Justifications

| Decision | Justification |
|----------|---------------|
| **DON'T spawn individual tile entities** | Rendering optimization requires Layer 3 control. bevy_ecs_tilemap uses TileStorage, Bevy native uses tilemap components. Individual entities with Transforms defeats batching. |
| **TileLayerData component with TileInstance grid** | Pre-processes GID → (tileset_handle, local_id, flip_flags). Layer 3 iterates once to spawn optimal rendering structure. Tileset handles included for collision shapes. |
| **Pre-compute shape vertices** | Both physics and debug rendering need Vec<Vec2>. Compute once in Layer 2, store in component. Avoids 2x computation. |
| **Components over SpawnContext in events** | Simpler lifetime management. Layer 3 uses standard ECS queries. Pre-computing data in components makes Layer 3 code cleaner. |
| **Remove TiledMapStorage** | Redundant. Relationships provide layer traversal. Component queries provide ID lookups (rare operation). TileInstance includes tileset handles directly. |
| **Both TiledLayerMapOf and TiledObjectMapOf** | Objects use `TiledObjectMapOf` (object → map). Layers use `TiledLayerMapOf` (layer → map). Allows both local queries and global queries. |
| **MergedProperties component** | Property inheritance computed once during spawning. Layer 3 queries for physics/gameplay properties without re-merging. |
| **Spawn object entities** | Objects have different shapes/behaviors, can't be batched like tiles. Each needs Transform, collision, custom logic. Individual entities make sense. |

---

## Entity Spawning Summary

| Layer Type | Layer 2 Spawns | Layer 2 Provides | Layer 3 Uses |
|------------|---------------|------------------|--------------|
| **Tile Layer** | ✅ Layer entity | TileLayerData component | Decides rendering (TileStorage, native tilemap, sprites) |
| **Object Layer** | ✅ Layer entity + object entities | TiledObject enum, pre-computed vertices | Adds rendering/physics to objects |
| **Image Layer** | ✅ Layer entity | ImageLayerData component | Adds Sprite/rendering |
| **Group Layer** | ✅ Layer entity | Hierarchical structure | Recursive traversal |

**Key Principle:** Layer 2 spawns entities for things that ARE entities (maps, layers, objects). Tiles are data in a grid, not individual game objects.

---

## Next Steps

1. Update `/Users/syynth/code/rs/bevy_tiled/docs/layer2-plan.md` with revised architecture:
   - Remove TiledMapStorage
   - Replace TiledTile component with TileLayerData
   - Add TileInstance struct (not a component)
   - Add ImageLayerData for image layers
   - Add pre-computed vertices to TiledObject
   - Add relationship components (TiledLayerMapOf, TiledObjectMapOf, etc.)
   - Update event system (TileLayerCreated, not TileCreated)
   - Update spawning phases

2. Verify this design with user before implementation
