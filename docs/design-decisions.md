# Design Decisions & Rationale

This document explains the key design decisions made in `bevy_tiled_topdown` and the reasoning behind them.

---

## Decision 1: Rendering-Agnostic Core

### The Decision

The core crate (`bevy_tiled_topdown`) has **no dependency** on any rendering library.

### What This Means

```rust
// ✅ Core provides these
pub struct TiledTile {
    pub tile_id: TileId,
    pub tileset_index: u32,
}

pub enum TiledObject {
    Rectangle { width: f32, height: f32 },
    // ...
}

// ❌ Core does NOT provide these
// - bevy_ecs_tilemap components
// - Sprite components
// - Mesh components
// - Rendering systems
```

### Rationale

**Problem with bevy_ecs_tiled:**
- Tightly coupled to `bevy_ecs_tilemap`
- Can't use alternative rendering approaches
- Forces all users to include rendering dependency

**Our solution:**
- Core provides **data structures** (`TiledTile`, `TiledObject`) and **events** (`TileCreated`)
- Rendering added via:
  1. Separate rendering plugin (e.g., `bevy_tiled_topdown_tilemap`)
  2. User observers that add rendering components
  3. Custom rendering implementations

**Example: User adds rendering**

```rust
fn on_tile_created(
    trigger: On<TiledEvent<TileCreated>>,
    mut commands: Commands,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
) {
    // Core gives us the tile data
    let tile_id = trigger.get_tile_id();
    let tileset = trigger.get_tileset_asset(&tileset_assets);

    // User decides how to render it
    commands.entity(trigger.entity).insert(/* rendering components */);
}
```

### Benefits

1. **Flexibility:** Users can choose rendering approach
2. **Smaller core:** No forced dependencies
3. **Future-proof:** Can adopt new rendering tech without breaking core
4. **Testing:** Can test core logic without rendering

### Trade-offs

- **More setup:** Users must add rendering plugin or write observers
- **Not batteries-included:** Core alone won't render anything
- **Documentation burden:** Must explain rendering integration

---

## Decision 2: Property Inheritance at Spawn Time

### The Decision

Custom property inheritance happens **during entity spawning** (Layer 2), **NOT during asset loading** (Layer 1).

### What This Means

**Layer 1 (Asset Loading):**
```rust
// Asset layer just parses and stores raw values
TiledTilesetAsset {
    tile_properties: HashMap<TileId, DeserializedProperties> {
        42: { health: 100, damage: 10 }
    }
}

TiledTemplateAsset {
    properties: { damage: 20, armor: 5 }
}

TiledMapAsset {
    properties.objects: HashMap<u32, DeserializedProperties> {
        7: { armor: 10 }
    }
}
// ↑ No merging happens here
```

**Layer 2 (Entity Spawning):**
```rust
// Spawning layer performs inheritance
fn spawn_object(object_id: 7) {
    let merged = DeserializedProperties::default()
        .merge_from(tile_properties)      // { health: 100, damage: 10 }
        .merge_from(template_properties)  // { health: 100, damage: 20, armor: 5 }
        .merge_from(object_properties);   // { health: 100, damage: 20, armor: 10 }

    entity.insert(merged.to_bundle());
}
```

### Rationale

**Problem with asset-time inheritance:**
- Assets are supposed to be **pure data**
- Merging at asset load time means asset contains processed, not raw, data
- Hard to debug (can't see which layer a property came from)
- Can't re-merge with different rules later
- Hot reloading issues (need to re-process inheritance)

**Our solution:**
- Assets store **raw, unmerged** properties per layer (tile, template, object)
- Spawning performs **merge** when creating entities
- Clean separation: data (Layer 1) vs. logic (Layer 2)

### Benefits

1. **Clean separation:** Assets are pure data, spawning is logic
2. **Debuggability:** Can inspect raw properties per layer
3. **Flexibility:** Can change merge rules without reloading assets
4. **Hot reloading:** Asset reload doesn't need to re-run inheritance
5. **Testing:** Can test inheritance algorithm separately from asset loading

### Trade-offs

- **Complexity:** Two-stage process (parse, then merge)
- **Memory:** Store properties at multiple layers (though minimal overhead)
- **Documentation:** Must explain the two stages

### Example: User Override

Because properties are raw in assets and merged at spawn:

```rust
fn custom_spawn_object(
    object: &Object,
    map_asset: &TiledMapAsset,
) {
    // User can override the merge order if needed
    let merged = DeserializedProperties::default()
        .merge_from(object_properties)   // ← Reversed!
        .merge_from(template_properties)
        .merge_from(tile_properties);    // ← Now tile wins

    // Or apply custom logic
    let mut merged = tile_properties.clone();
    if special_condition {
        merged.merge_from(template_properties);
    }
}
```

---

## Decision 3: First-Class Asset Support for All Tiled Files

### The Decision

All Tiled file types (`.tmx`, `.tsx`, `.tx`, `.world`) are **Bevy assets** with proper dependency tracking.

### What This Means

**bevy_ecs_tiled approach:**
```rust
TiledMapAsset {
    map: tiled::Map,
    tilesets: HashMap<String, TiledMapTileset>,  // Embedded data
    // ↑ Tilesets copied into map asset
}
```

**Our approach:**
```rust
TiledMapAsset {
    map: tiled::Map,
    tilesets: HashMap<u32, TilesetReference {
        handle: Handle<TiledTilesetAsset>,  // ← Bevy asset handle
        first_gid: u32,
    }>,
    templates: HashMap<String, Handle<TiledTemplateAsset>>,  // ← Asset handles
    images: HashMap<u32, Handle<Image>>,                     // ← Asset handles
}

// Separate assets
TiledTilesetAsset { /* ... */ }
TiledTemplateAsset { /* ... */ }
```

### Rationale

**Problems with embedded data:**
1. **No sharing:** Multiple maps using same tileset load it multiple times
2. **No hot reload:** Can't reload tileset without reloading entire map
3. **No dependency tracking:** Bevy doesn't know about tileset dependencies
4. **Large assets:** Map assets contain duplicate tileset data
5. **Templates not assets:** Can't load templates standalone

**Our solution:**
- Tilesets are **separate assets** with unique asset IDs
- Maps reference tilesets via **handles**
- Templates are **loadable assets**
- Bevy's asset system handles:
  - Dependency tracking
  - Hot reloading
  - Deduplication
  - Lifecycle management

### Benefits

1. **Sharing:** Multiple maps can share one tileset asset
2. **Hot reload:** Change tileset, all maps using it update
3. **Memory:** Tilesets loaded once, referenced many times
4. **Templates:** Can spawn template as standalone entity
5. **Standard Bevy:** Uses standard asset system patterns

### Trade-offs

- **More asset types:** 4 asset types instead of 2
- **More loaders:** 4 loaders instead of 2
- **Handle indirection:** Access tileset via `tileset_assets.get(&handle)`

### Example: Hot Reload

```rust
// User edits "tileset.tsx" in Tiled
// Bevy detects file change
// TiledTilesetAssetLoader reloads tileset
// All maps using this tileset automatically update
// (because they reference it via Handle)
```

### Example: Template as Standalone Entity

```rust
// NEW: Spawn template without a map
commands.spawn(TiledTemplate {
    handle: asset_server.load("templates/chest.tx"),
    position: Some(Vec2::new(100.0, 200.0)),
    rotation: None,
});
// ↑ Not possible in bevy_ecs_tiled
```

---

## Decision 4: Preserve Raw tiled Crate Types in Assets

### The Decision

Asset structs **preserve** the raw `tiled::Map`, `tiled::Tileset`, etc. instead of converting to custom types.

### What This Means

```rust
// ✅ Our approach
#[derive(Asset)]
pub struct TiledMapAsset {
    pub map: tiled::Map,  // ← Preserved!
    // ... Bevy-specific additions
}

// ❌ Alternative approach (not used)
pub struct TiledMapAsset {
    pub width: u32,
    pub height: u32,
    pub orientation: CustomOrientation,
    // ... re-implement all fields
}
```

### Rationale

**Why preserve:**
1. **Full data access:** Users can access ANY field from tiled crate
2. **Future-proof:** New tiled crate features work automatically
3. **Less code:** Don't re-implement tiled crate's types
4. **Compatibility:** Easy to port code from direct tiled usage
5. **Trust the source:** tiled crate is the canonical Tiled format implementation

**What we add:**
- **Bevy integration:** Handles, components, events
- **Processed data:** Pre-calculated tilemap sizes, bounding boxes
- **Custom properties:** Parsed and typed properties

### Benefits

1. **Simplicity:** Less code to maintain
2. **Completeness:** No missing fields
3. **Updates:** New tiled crate versions automatically supported
4. **Familiarity:** Users familiar with tiled crate feel at home

### Trade-offs

- **Dependency:** Tightly coupled to tiled crate (but we depend on it anyway)
- **Ownership:** Need `Arc<tiled::Tileset>` for sharing (minor)

### Example: Accessing Tiled Data

```rust
fn system(map_assets: Res<Assets<TiledMapAsset>>) {
    for map_asset in map_assets.iter() {
        // Direct access to tiled crate data
        let orientation = map_asset.map.orientation;
        let background_color = map_asset.map.background_color;
        let custom_property = map_asset.map.properties.get("my_prop");

        // All tiled::Map methods available
        for layer in map_asset.map.layers() {
            // ...
        }
    }
}
```

---

## Decision 5: Handle-Based References Instead of Arc

### The Decision

Use Bevy's **`Handle<T>`** for asset references instead of Rust's `Arc<T>`.

### What This Means

**tiled crate:**
```rust
pub struct Map {
    tilesets: Vec<Arc<Tileset>>,  // ← Arc-based sharing
}
```

**Our assets:**
```rust
pub struct TiledMapAsset {
    tilesets: HashMap<u32, TilesetReference {
        handle: Handle<TiledTilesetAsset>,  // ← Handle-based
    }>,
}
```

### Rationale

**Arc approach:**
- Manual reference counting
- No hot reload support
- No asset lifecycle integration
- No dependency tracking

**Handle approach:**
- Bevy manages lifecycle
- Hot reload supported
- Automatic dependency tracking
- Strong/weak handle variants
- Asset event integration

### Benefits

1. **Hot reload:** Bevy detects file changes and reloads
2. **Lifecycle:** Assets unload when no handles remain
3. **Events:** `AssetEvent::Added`, `AssetEvent::Modified`, `AssetEvent::Removed`
4. **Debugging:** Can inspect asset state via Bevy tools

---

## Decision 6: Dual Representation (Asset + Component)

### The Decision

Data exists in **two forms:**
1. **Asset form:** Raw data loaded from files (Layer 1)
2. **Component form:** Data on entities in ECS (Layer 2)

### What This Means

**Map:**
- **Asset:** `TiledMapAsset` (contains `tiled::Map` + handles)
- **Component:** `TiledMap` (just contains `Handle<TiledMapAsset>`)
- **Storage:** `TiledMapStorage` component (Tiled ID ↔ Entity mapping)

**Object:**
- **Asset:** Data in `map_asset.properties.objects: HashMap<u32, DeserializedProperties>`
- **Component:** `TiledObject` enum (geometry), plus custom property components

**Tile:**
- **Asset:** Data in `tileset_asset.tile_properties: HashMap<TileId, DeserializedProperties>`
- **Component:** `TiledTile` (reference), plus custom property components

### Rationale

**Why two representations:**
1. **Assets are shared:** Multiple entities may reference same asset data
2. **Entities are instances:** Each entity is a specific occurrence
3. **Different lifecycles:** Assets loaded/unloaded separately from entities
4. **ECS patterns:** Components are copyable/cloneable data on entities

**Bridges between forms:**
- `TiledMapStorage` maps Tiled IDs to Entities
- Events carry both entity and asset context
- Spawning converts asset data to components

### Benefits

1. **Efficient:** Shared data in assets, instance data on entities
2. **Standard Bevy:** Follows Bevy's asset + component pattern (like `Scene` + `SceneInstance`)
3. **Query-able:** Can query entities by components
4. **Flexible:** Can spawn multiple instances from one asset

### Example

```rust
// One asset
let map_asset = TiledMapAsset { /* ... */ };

// Multiple entities using it
commands.spawn(TiledMap { handle: map_asset.handle.clone() });
commands.spawn(TiledMap { handle: map_asset.handle.clone() });
// ↑ Two map instances from one asset
```

---

## Decision 7: Events for Reactivity

### The Decision

Use **Bevy observers** and **entity events** for extensibility instead of hooks/callbacks.

### What This Means

```rust
// ✅ Our approach (observers)
fn on_object_created(
    trigger: On<TiledEvent<ObjectCreated>>,
    q_doors: Query<&Door>,
) {
    if let Ok(door) = q_doors.get(trigger.entity) {
        // React to door creation
    }
}

app.add_observer(on_object_created);

// ❌ Alternative (callbacks)
TiledPluginConfig {
    on_object_created: |entity, object| {
        // Callback
    },
}
```

### Rationale

**Observer benefits:**
1. **Decoupled:** Multiple systems can react to same event
2. **ECS-native:** Uses standard Bevy patterns
3. **Type-safe:** Events carry typed context
4. **Auto-propagation:** Events bubble through entity hierarchy
5. **Flexible:** Users can add observers without modifying core

**Event bubbling:**
```
Tile Entity
  ↓ TiledEvent<TileCreated>
Tilemap Entity
  ↓ (propagates)
Layer Entity
  ↓ (propagates)
Map Entity
  ↓ (propagates)

Observer can be attached at any level
```

### Benefits

1. **Extensibility:** Users add observers for custom behavior
2. **Composability:** Multiple plugins can react independently
3. **Standard:** Uses Bevy 0.17's observer system
4. **Context:** Events carry full context (entity, origin, asset IDs)

---

## Decision 8: Storage Pattern (Bidirectional Mapping)

### The Decision

Maintain **bidirectional mapping** between Tiled IDs and Bevy Entities via `TiledMapStorage` component.

### What This Means

```rust
#[derive(Component)]
pub struct TiledMapStorage {
    layers: HashMap<u32, Entity>,                      // Tiled layer ID → Entity
    objects: HashMap<u32, Entity>,                     // Tiled object ID → Entity
    tiles: HashMap<(u32, TileId), Vec<Entity>>,       // (tileset, tile) → Entities
}

impl TiledMapStorage {
    // Tiled ID → Entity
    pub fn get_layer_entity(&self, layer_id: u32) -> Option<Entity>;
    pub fn get_object_entity(&self, object_id: u32) -> Option<Entity>;

    // Entity → Tiled data
    pub fn get_layer<'a>(&self, map: &'a Map, entity: Entity) -> Option<Layer<'a>>;
    pub fn get_object<'a>(&self, map: &'a Map, entity: Entity) -> Option<Object<'a>>;
}
```

### Rationale

**Use cases:**
1. **Find entity from Tiled ID:** "What entity is object 42?"
2. **Find Tiled data from entity:** "What's the object data for this entity?"
3. **Debugging:** Correlate entities with Tiled editor data
4. **Scripting:** Reference objects by Tiled name/ID in scripts

### Benefits

1. **Query by ID:** Can find entities using Tiled IDs
2. **Query by entity:** Can look up Tiled data from entities
3. **Preserved from bevy_ecs_tiled:** Users migrating will find familiar API

---

## Summary of Design Principles

1. **Clean Layering:** Asset loading → Entity spawning → Reactive logic
2. **Data-First:** Assets are pure data, processing happens at spawn
3. **Bevy-Native:** Use Bevy patterns (assets, handles, components, events)
4. **Rendering-Agnostic:** Core has no rendering dependency
5. **Preserve Raw Data:** Keep `tiled::T` types intact, add Bevy integration alongside
6. **Handle-Based:** Use `Handle<T>` for references, not `Arc<T>`
7. **Event-Driven:** Observers for extensibility
8. **First-Class Everything:** All Tiled file types are Bevy assets
