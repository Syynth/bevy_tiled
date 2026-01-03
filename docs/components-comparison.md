# Components & Resources: bevy_ecs_tiled vs bevy_tiled_topdown

This document provides a comprehensive comparison of all components and resources between `bevy_ecs_tiled` and `bevy_tiled_topdown`, indicating which will be preserved, modified, or removed.

---

## Legend

- ✅ **KEEP** - Preserve as-is or with minimal changes
- 🔄 **MODIFY** - Keep but change structure/behavior
- ❌ **REMOVE** - Not needed in new design
- ➕ **NEW** - New component/resource in bevy_tiled_topdown

---

## COMPONENTS

### Map-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledMap** | `Handle<TiledMapAsset>` | `Handle<TiledMapAsset>` | ✅ **KEEP** | Same API, different asset structure |
| **TiledMapLayerZOffset** | `f32` (default: 100.0) | `f32` (default: 100.0) | ✅ **KEEP** | Unchanged |
| **TiledMapImageRepeatMargin** | `u32` (default: 1) | ❌ **REMOVE** | ❌ **REMOVE** | Rendering-specific, move to rendering plugin |
| **TiledMapReference** | `Entity` (parent map) | `Entity` (parent map) | ✅ **KEEP** | Unchanged |
| **RespawnTiledMap** | Marker | Marker | ✅ **KEEP** | Hot reload support unchanged |

**Changes:**
- `TiledMapImageRepeatMargin` removed from core (rendering-agnostic design)
- Auto-required components may differ based on Bevy 0.17 requirements

---

### Storage Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledMapStorage** | HashMaps for layers, objects, tiles | Same structure | ✅ **KEEP** | Bidirectional mapping preserved |
| **TiledWorldStorage** | HashMap for maps | Same structure | ✅ **KEEP** | Bidirectional mapping preserved |

**Changes:**
- None, these are core to the design and work well

---

### Layer-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledLayer** | Enum: Tiles, Objects, Image, Group | `{ layer_id: u32, layer_type: TiledLayerType }` | 🔄 **MODIFY** | Add `layer_id` field for easier lookup |
| **TiledLayerParallax** | `{ parallax_x, parallax_y, base_position }` | Same structure | ✅ **KEEP** | Unchanged |
| **TiledParallaxCamera** | Marker | Marker | ✅ **KEEP** | Unchanged |

**New enum:**
```rust
// bevy_tiled_topdown
pub enum TiledLayerType {
    Tiles,
    Objects,
    Image,
    Group,
}
```

**Changes:**
- `TiledLayer` now stores both layer ID and type (easier queries)
- Parallax support unchanged

---

### Object-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledObject** | Enum with geometry variants | Same enum | ✅ **KEEP** | Unchanged |
| **TiledObjectVisualOf** | Relationship component | Same structure | ✅ **KEEP** | Relationship preserved |
| **TiledObjectVisuals** | Relationship target | Same structure | ✅ **KEEP** | Relationship preserved |

**Changes:**
- None, object representation works well

---

### Tile-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledTile** | Marker (no data) | `{ tile_id: TileId, tileset_index: u32 }` | 🔄 **MODIFY** | Add data for easier lookup |
| **TiledTilemap** | Marker (no data) | `{ tileset_index: u32 }` | 🔄 **MODIFY** | Add tileset reference |

**Changes:**
- Add data to marker components for easier queries
- Enables querying tiles by ID without accessing storage

---

### World-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledWorld** | `Handle<TiledWorldAsset>` | `Handle<TiledWorldAsset>` | ✅ **KEEP** | Same API, different asset structure |
| **RespawnTiledWorld** | Marker | Marker | ✅ **KEEP** | Hot reload support unchanged |
| **TiledWorldChunking** | `Option<Vec2>` | Same structure | ✅ **KEEP** | Chunking unchanged |

**Changes:**
- None, world system works well

---

### Image-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledImage** | `{ base_position, base_size }` | ❌ **REMOVE** | ❌ **REMOVE** | Rendering-specific, move to rendering plugin |

**Changes:**
- Remove from core (rendering-agnostic design)
- Rendering plugin can add if needed

---

### Animation Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledAnimation** | `{ start, end, timer }` | `{ frames: Vec<Frame>, current_frame, elapsed }` | 🔄 **MODIFY** | More flexible frame data |

**New structure:**
```rust
pub struct TiledAnimation {
    pub frames: Vec<TiledAnimationFrame>,
    pub current_frame: usize,
    pub elapsed: f32,
}

pub struct TiledAnimationFrame {
    pub tile_id: TileId,
    pub duration: f32,
}
```

**Changes:**
- Store full frame data (not just range)
- Supports variable frame durations
- Rendering-agnostic (no Sprite dependency in core)

---

### Physics-Related Components

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledColliderSource** | Enum: TilesLayer, Object | Same enum | ✅ **KEEP** | Unchanged |
| **TiledColliderOf** | Relationship component | Same structure | ✅ **KEEP** | Relationship preserved |
| **TiledColliders** | Relationship target | Same structure | ✅ **KEEP** | Relationship preserved |
| **TiledColliderPolygons** | `geo::MultiPolygon<f32>` | Same structure | ✅ **KEEP** | Unchanged |
| **TiledPhysicsSettings<T>** | Generic over backend | Same structure | ✅ **KEEP** | Unchanged (Avian only) |

**Changes:**
- Drop Rapier backend support (Avian only)
- Otherwise unchanged

---

### Template Components (NEW)

| Component | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-----------|----------------|-------------------|--------|-------|
| **TiledTemplate** | ❌ Not supported | `{ handle: Handle<...>, position, rotation }` | ➕ **NEW** | Spawn templates standalone |

**New component:**
```rust
#[derive(Component, Reflect)]
pub struct TiledTemplate {
    pub handle: Handle<TiledTemplateAsset>,
    pub position: Option<Vec2>,
    pub rotation: Option<f32>,
}
```

**Purpose:** Load and spawn Tiled templates as standalone entities (not part of a map)

---

## RESOURCES

### Configuration Resources

| Resource | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|----------|----------------|-------------------|--------|-------|
| **TiledPluginConfig** | `{ export_file, filter }` | Same structure | ✅ **KEEP** | Custom properties export unchanged |
| **TiledResourceCache** | `Arc<RwLock<...>>` | Same structure | ✅ **KEEP** | Internal caching unchanged |

**Changes:**
- None, configuration works well

---

### Debug Resources (Feature-Gated)

| Resource | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|----------|----------------|-------------------|--------|-------|
| **TiledDebugTilesConfig** | `{ color, font, z_offset, scale }` | ❌ **REMOVE** | ❌ **REMOVE** | Rendering-specific debug |
| **TiledDebugObjectsConfig** | `{ colors, arrow_length }` | ❌ **REMOVE** | ❌ **REMOVE** | Rendering-specific debug |
| **TiledDebugWorldChunkConfig** | `{ world_chunk_color, maps_colors }` | ❌ **REMOVE** | ❌ **REMOVE** | Rendering-specific debug |

**Changes:**
- Remove rendering-specific debug from core
- Debug visualization can be in separate plugin

---

### Rendering Resources (NEW APPROACH)

| Resource | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|----------|----------------|-------------------|--------|-------|
| **TilemapRenderSettings** | In core | ❌ **REMOVE** | ❌ **REMOVE** | Move to rendering plugin |
| **TilemapAnchor** | In core | ❌ **REMOVE** | ❌ **REMOVE** | Move to rendering plugin |

**Changes:**
- All rendering-related resources removed from core
- Separate rendering plugin provides these

---

## EVENT TYPES

| Event | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|-------|----------------|-------------------|--------|-------|
| **WorldCreated** | ✅ | ✅ | ✅ **KEEP** | Unchanged |
| **MapCreated** | ✅ | ✅ | ✅ **KEEP** | Unchanged |
| **LayerCreated** | ✅ | ✅ | ✅ **KEEP** | Unchanged |
| **TilemapCreated** | ✅ | ❌ | ❌ **REMOVE** | Rendering-specific, optional in rendering plugin |
| **TileCreated** | ✅ | ✅ | ✅ **KEEP** | Unchanged |
| **ObjectCreated** | ✅ | ✅ | ✅ **KEEP** | Unchanged |
| **ColliderCreated** | ✅ (feature-gated) | ✅ (feature-gated) | ✅ **KEEP** | Unchanged |
| **TemplateSpawned** | ❌ | ✅ | ➕ **NEW** | For standalone template spawning |

**Changes:**
- Remove `TilemapCreated` (rendering-specific)
- Add `TemplateSpawned` for new template feature

---

## ADDITIONAL TYPES

### TiledFilter

| Type | bevy_ecs_tiled | bevy_tiled_topdown | Status | Notes |
|------|----------------|-------------------|--------|-------|
| **TiledFilter** | Enum: All, Names, RegexSet, None | Same enum | ✅ **KEEP** | Unchanged |

**Changes:**
- None, filtering utility is perfect as-is

---

## SUMMARY BY CATEGORY

### Components: 26 → ~22

**Kept (unchanged):** 16
- TiledMap, TiledWorld, TiledMapReference, RespawnTiledMap, RespawnTiledWorld
- TiledMapStorage, TiledWorldStorage
- TiledLayerParallax, TiledParallaxCamera
- TiledObject, TiledObjectVisualOf, TiledObjectVisuals
- TiledWorldChunking
- TiledColliderSource, TiledColliderOf, TiledColliders, TiledColliderPolygons

**Modified:** 4
- TiledLayer (add layer_id field)
- TiledTile (add tile data)
- TiledTilemap (add tileset_index)
- TiledAnimation (more flexible frame data)

**Removed:** 4
- TiledMapImageRepeatMargin (rendering-specific)
- TiledImage (rendering-specific)
- Debug visualization components (rendering-specific)

**Added:** 2
- TiledTemplate (new feature)
- TiledPhysicsSettings (moved from feature to core API)

---

### Resources: 5 → 2

**Kept:** 2
- TiledPluginConfig
- TiledResourceCache (internal)

**Removed:** 3
- All debug resources (rendering-specific)

**Added:** 0

---

### Events: 7 → 7

**Kept:** 6
- WorldCreated, MapCreated, LayerCreated, TileCreated, ObjectCreated, ColliderCreated

**Removed:** 1
- TilemapCreated (rendering-specific)

**Added:** 1
- TemplateSpawned (new feature)

---

## MIGRATION GUIDE

### Component Renames/Changes

**None** - All kept components have the same name and similar structure

### Removed Components

If you were using these, you'll need to:

1. **TiledMapImageRepeatMargin** → Use rendering plugin's equivalent
2. **TiledImage** → Use rendering plugin's sprite components
3. **Debug resources** → Use rendering plugin's debug features

### New Components

**TiledTemplate** - Available for spawning templates:
```rust
commands.spawn(TiledTemplate {
    handle: asset_server.load("template.tx"),
    position: Some(Vec2::new(100.0, 200.0)),
    rotation: None,
});
```

### Modified Components

**TiledLayer** - Now has `layer_id` field:
```rust
// Old
match tiled_layer {
    TiledLayer::Tiles => { /* ... */ }
}

// New
if tiled_layer.layer_type == TiledLayerType::Tiles {
    let layer_id = tiled_layer.layer_id;
    // ...
}
```

**TiledTile** - Now has data:
```rust
// Old
// Had to use TiledMapStorage to get tile info

// New
fn system(q_tiles: Query<&TiledTile>) {
    for tile in q_tiles.iter() {
        let tile_id = tile.tile_id;
        let tileset_index = tile.tileset_index;
        // Direct access without storage lookup
    }
}
```

---

## COMPONENT USAGE PATTERNS

### Query Patterns

**Find all objects in a specific layer:**
```rust
// Old (bevy_ecs_tiled)
fn system(
    map_storage: Query<&TiledMapStorage>,
    map_assets: Res<Assets<TiledMapAsset>>,
) {
    // Complex storage lookups
}

// New (bevy_tiled_topdown)
fn system(
    q_layers: Query<(Entity, &TiledLayer), With<TiledMapReference>>,
    q_objects: Query<&TiledObject, With<Parent>>,
) {
    for (layer_entity, layer) in q_layers.iter() {
        if layer.layer_type == TiledLayerType::Objects {
            // Query objects by parent
        }
    }
}
```

**Find all tiles with specific ID:**
```rust
// Old (bevy_ecs_tiled)
fn system(map_storage: Query<&TiledMapStorage>) {
    for storage in map_storage.iter() {
        let entities = storage.get_tile_entities(tileset_id, tile_id);
    }
}

// New (bevy_tiled_topdown)
fn system(q_tiles: Query<(Entity, &TiledTile)>) {
    for (entity, tile) in q_tiles.iter() {
        if tile.tile_id == target_id && tile.tileset_index == tileset_idx {
            // Found matching tile
        }
    }
}
```

**Both approaches work** - storage still available for bidirectional lookup, but simple queries are easier now.

---

## DESIGN RATIONALE

### Why Remove Rendering Components?

**Goal:** Rendering-agnostic core

**Benefits:**
1. Core has no `bevy_ecs_tilemap` dependency
2. Users can choose rendering approach
3. Smaller core crate
4. Future-proof for new rendering tech

**Trade-off:**
- Users must add rendering plugin or observers
- Not "batteries included" anymore

### Why Add Data to Marker Components?

**Goal:** Easier querying without storage lookups

**Benefits:**
1. Simple queries don't need `TiledMapStorage`
2. Direct ECS queries are more ergonomic
3. Better performance for common queries

**Trade-off:**
- Slightly more memory per entity
- Data duplication (also in storage)

**Decision:** Worth it for ergonomics

### Why Keep Storage Components?

**Goal:** Preserve bidirectional mapping

**Benefits:**
1. Can lookup by Tiled ID (from scripts/external tools)
2. Can get Tiled data from entity
3. Debugging and editor tools need this

**Trade-off:**
- Memory overhead
- Complexity

**Decision:** Essential for Tiled integration

---

## COMPARISON TABLE: FULL INVENTORY

| Component | ecs_tiled | tiled_topdown | Change |
|-----------|-----------|---------------|--------|
| TiledMap | ✅ | ✅ | Same |
| TiledMapLayerZOffset | ✅ | ✅ | Same |
| TiledMapImageRepeatMargin | ✅ | ❌ | Removed |
| TiledMapReference | ✅ | ✅ | Same |
| RespawnTiledMap | ✅ | ✅ | Same |
| TiledMapStorage | ✅ | ✅ | Same |
| TiledWorldStorage | ✅ | ✅ | Same |
| TiledLayer | ✅ | ✅ | Modified (add layer_id) |
| TiledLayerParallax | ✅ | ✅ | Same |
| TiledParallaxCamera | ✅ | ✅ | Same |
| TiledObject | ✅ | ✅ | Same |
| TiledObjectVisualOf | ✅ | ✅ | Same |
| TiledObjectVisuals | ✅ | ✅ | Same |
| TiledTile | ✅ | ✅ | Modified (add data) |
| TiledTilemap | ✅ | ✅ | Modified (add tileset_index) |
| TiledWorld | ✅ | ✅ | Same |
| RespawnTiledWorld | ✅ | ✅ | Same |
| TiledWorldChunking | ✅ | ✅ | Same |
| TiledImage | ✅ | ❌ | Removed |
| TiledAnimation | ✅ | ✅ | Modified (frame data) |
| TiledColliderSource | ✅ | ✅ | Same |
| TiledColliderOf | ✅ | ✅ | Same |
| TiledColliders | ✅ | ✅ | Same |
| TiledColliderPolygons | ✅ | ✅ | Same |
| TiledPhysicsSettings | ✅ | ✅ | Same (Avian only) |
| TiledTemplate | ❌ | ✅ | **NEW** |
| TiledEvent<E> | ✅ | ✅ | Same |

**Total:** 26 components → 22 components (4 removed, 1 added)

---

## RESOURCES TABLE

| Resource | ecs_tiled | tiled_topdown | Change |
|----------|-----------|---------------|--------|
| TiledPluginConfig | ✅ | ✅ | Same |
| TiledResourceCache | ✅ | ✅ | Same |
| TiledDebugTilesConfig | ✅ | ❌ | Removed |
| TiledDebugObjectsConfig | ✅ | ❌ | Removed |
| TiledDebugWorldChunkConfig | ✅ | ❌ | Removed |

**Total:** 5 resources → 2 resources (3 removed)

---

## CONCLUSION

**Key Takeaways:**

1. **Most components preserved** - 16 of 26 unchanged
2. **Rendering removed** - 4 components, 3 resources
3. **Ergonomics improved** - Added data to marker components
4. **New feature** - Template spawning
5. **Physics simplified** - Avian only
6. **Core principles maintained** - Storage, events, relationships

**Migration impact:**
- **Low** for most users (core API unchanged)
- **Medium** if using rendering features (need plugin)
- **High** if customizing rendering (different extension points)
