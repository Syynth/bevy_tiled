# Type Mapping: tiled crate → bevy_tiled_topdown

This document compares the type structure of the `tiled` crate (v0.15) with the proposed `bevy_tiled_topdown` asset types, explaining the key differences and design rationale.

## Core Philosophy

**tiled crate:** Provides data structures that mirror the TMX/TSX/TX file formats with minimal processing. Uses `Arc<T>` for shared ownership and lifetimes for borrowing.

**bevy_tiled_topdown:** Wraps tiled crate types in Bevy assets with handle-based references. Separates raw data (asset layer) from ECS entities (spawning layer).

---

## Map Types

### tiled::Map

```rust
// From tiled crate (map.rs:28-76)
pub struct Map {
    version: String,
    pub source: PathBuf,
    pub orientation: Orientation,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub hex_side_length: Option<i32>,
    pub stagger_axis: StaggerAxis,
    pub stagger_index: StaggerIndex,
    tilesets: Vec<Arc<Tileset>>,        // ← Owned via Arc
    layers: Vec<LayerData>,             // ← Owned
    pub properties: Properties,
    pub background_color: Option<Color>,
    infinite: bool,
    pub user_type: Option<String>,
}
```

**Key characteristics:**
- Contains **owned** `Arc<Tileset>` references (shared ownership)
- Contains **owned** `LayerData` (no borrowing)
- Uses `Arc` to allow multiple maps to share tilesets
- No lifetime parameters (all data is owned or Arc-wrapped)

### TiledMapAsset (Our Design)

```rust
#[derive(TypePath, Asset, Debug)]
pub struct TiledMapAsset {
    /// The raw Tiled map data (PRESERVE AS-IS)
    pub map: tiled::Map,

    // ===== BEVY ASSET REFERENCES =====
    /// Tileset handles (Bevy asset system)
    /// Key: Tileset index in map
    pub tilesets: HashMap<u32, TilesetReference>,

    /// Template handles (Bevy asset system)
    /// Key: Template source path
    pub templates: HashMap<String, Handle<TiledTemplateAsset>>,

    /// Image layer images (Bevy asset system)
    /// Key: Layer ID
    pub images: HashMap<u32, Handle<Image>>,

    // ===== PROCESSED DATA FOR BEVY =====
    /// Map size in tiles (for tilemap systems)
    pub tilemap_size: TilemapSize,

    /// Largest tile size across all tilesets
    pub largest_tile_size: TilemapTileSize,

    /// Map bounding box in pixels
    pub rect: Rect,

    // ===== INFINITE MAP SUPPORT =====
    pub(crate) tiled_offset: Vec2,
    pub(crate) topleft_chunk: (i32, i32),
    pub(crate) bottomright_chunk: (i32, i32),

    // ===== CUSTOM PROPERTIES =====
    #[cfg(feature = "user_properties")]
    pub properties: DeserializedMapProperties,
}

#[derive(Debug, Clone)]
pub struct TilesetReference {
    /// Bevy asset handle to the tileset
    pub handle: Handle<TiledTilesetAsset>,
    /// First GID of this tileset in the map
    pub first_gid: u32,
}
```

**Design differences:**

| Aspect | tiled::Map | TiledMapAsset |
|--------|------------|---------------|
| **Tilesets** | `Vec<Arc<Tileset>>` (owned, shared) | `HashMap<u32, TilesetReference>` (Bevy handles) |
| **Templates** | Not tracked at map level | `HashMap<String, Handle<TiledTemplateAsset>>` |
| **Images** | Not tracked at map level | `HashMap<u32, Handle<Image>>` (image layers) |
| **Ownership** | Arc-based shared ownership | Bevy asset handle-based |
| **Lifecycle** | Manual via Arc refcount | Bevy asset system |
| **Hot reload** | Not supported | Supported via Bevy |
| **Dependencies** | Manually loaded via ResourceCache | Auto-tracked via LoadContext |

**Rationale:**
1. **Preserve raw data:** Keep `tiled::Map` as-is for full access to original data
2. **Add Bevy integration:** Parallel handle-based references for Bevy asset system
3. **Dependency tracking:** Bevy's asset system tracks all dependencies automatically
4. **Hot reloading:** Handle-based references enable asset hot reloading
5. **First-class templates:** Templates become loadable assets, not just inline data

---

## Tileset Types

### tiled::Tileset

```rust
// From tiled crate (tileset.rs:18-74)
pub struct Tileset {
    pub source: PathBuf,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tilecount: u32,
    pub columns: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub image: Option<Image>,                     // ← Image path, not loaded
    tiles: HashMap<TileId, TileData>,             // ← Per-tile data
    pub wang_sets: Vec<WangSet>,
    pub properties: Properties,
    pub user_type: Option<String>,
}

// Image is just metadata (image.rs)
pub struct Image {
    pub source: PathBuf,    // ← Just a path, not a loaded texture
    pub width: i32,
    pub height: i32,
    pub trans: Option<Color>,
}
```

**Key characteristics:**
- Contains image **path**, not loaded texture data
- No lifetime parameters (all owned)
- `tiles` HashMap contains per-tile data (animations, properties, etc.)

### TiledTilesetAsset (Our Design)

```rust
#[derive(TypePath, Asset, Debug, Clone)]
pub struct TiledTilesetAsset {
    /// The raw Tiled tileset data (PRESERVE AS-IS)
    pub tileset: Arc<tiled::Tileset>,

    // ===== LOADED TEXTURE DATA =====
    /// Tilemap texture info (for rendering systems)
    pub tilemap_texture: TilemapTexture,

    /// Texture atlas layout (for single-image tilesets)
    pub texture_atlas_layout: Option<Handle<TextureAtlasLayout>>,

    /// Loaded image handle (for single-image tilesets)
    pub image: Option<Handle<Image>>,

    /// Loaded tile images (for image collection tilesets)
    /// Key: TileId, Value: Image handle
    pub tile_images: HashMap<tiled::TileId, Handle<Image>>,

    // ===== METADATA =====
    /// Whether this tileset can be used for tile layers
    /// (all tiles must have same dimensions)
    pub usable_for_tiles_layer: bool,

    // ===== CUSTOM PROPERTIES =====
    #[cfg(feature = "user_properties")]
    pub tile_properties: HashMap<tiled::TileId, DeserializedProperties>,
}
```

**Design differences:**

| Aspect | tiled::Tileset | TiledTilesetAsset |
|--------|----------------|-------------------|
| **Tileset data** | Owns the data directly | `Arc<tiled::Tileset>` (can share) |
| **Images** | `Option<Image>` (path only) | `Option<Handle<Image>>` (loaded texture) |
| **Tile images** | Paths in `TileData.image` | `HashMap<TileId, Handle<Image>>` (loaded) |
| **Texture atlas** | Not present | `Handle<TextureAtlasLayout>` (created) |
| **Loading** | Parse-only, no texture loading | Loads all textures as dependencies |
| **Custom properties** | `Properties` (raw) | `DeserializedProperties` (typed) |
| **Validation** | None | `usable_for_tiles_layer` flag |

**Rationale:**
1. **Wrap, don't replace:** Use `Arc<tiled::Tileset>` to preserve original data
2. **Load textures:** Convert image paths to loaded `Handle<Image>`
3. **Prepare for rendering:** Create texture atlases for rendering systems
4. **Type custom properties:** Parse properties into typed components
5. **Validate:** Check if tileset is suitable for tile layers

---

## Template Types

### tiled::Template (also called ObjectTemplate)

```rust
// From tiled crate (template.rs:16-24)
pub struct Template {
    pub source: PathBuf,
    pub tileset: Option<Arc<Tileset>>,    // ← Shared tileset
    pub object: ObjectData,                // ← Object definition
}
```

**Key characteristics:**
- Minimal structure: just tileset reference + object data
- Uses `Arc<Tileset>` for shared ownership
- `ObjectData` contains all object properties and geometry
- No properties at template level (properties are on the object)

### TiledTemplateAsset (Our Design)

```rust
#[derive(TypePath, Asset, Debug, Clone)]
pub struct TiledTemplateAsset {
    /// The raw Tiled template data (PRESERVE AS-IS)
    pub template: tiled::Template,

    // ===== BEVY ASSET REFERENCES =====
    /// Reference to the tileset (Bevy asset)
    pub tileset: Option<Handle<TiledTilesetAsset>>,

    // ===== CUSTOM PROPERTIES =====
    #[cfg(feature = "user_properties")]
    pub properties: DeserializedProperties,
}
```

**Design differences:**

| Aspect | tiled::Template | TiledTemplateAsset |
|--------|-----------------|---------------------|
| **Template data** | Owns directly | Preserved as `tiled::Template` |
| **Tileset** | `Option<Arc<Tileset>>` | `Option<Handle<TiledTilesetAsset>>` |
| **Object data** | In `template.object` | In `template.object` |
| **Custom properties** | In `object.properties` | Extracted to `properties` field |
| **Loading** | No dependency tracking | Tileset loaded as dependency |
| **Spawning** | Not designed for it | Can spawn standalone via `TiledTemplate` component |

**Rationale:**
1. **Bevy asset:** Templates become first-class loadable assets
2. **Handle references:** Tileset is a Bevy asset dependency
3. **Extract properties:** Pre-parse properties for spawn-time inheritance
4. **Standalone spawning:** Enable `TiledTemplate` component for non-map objects

---

## World Types

### tiled::World

```rust
// From tiled crate (world.rs)
pub struct World {
    pub source: PathBuf,
    pub maps: Vec<WorldMap>,
    pub patterns: Vec<WorldPattern>,
}

pub struct WorldMap {
    pub file_name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
```

**Key characteristics:**
- Contains map **metadata**, not loaded maps
- Maps are referenced by file name
- No actual map data loaded

### TiledWorldAsset (Our Design)

```rust
#[derive(TypePath, Asset, Debug)]
pub struct TiledWorldAsset {
    /// The raw Tiled world data (PRESERVE AS-IS)
    pub world: tiled::World,

    // ===== BEVY ASSET REFERENCES =====
    /// Map handles (Bevy asset system)
    /// Key: Map path from world
    pub maps: HashMap<String, Handle<TiledMapAsset>>,
}
```

**Design differences:**

| Aspect | tiled::World | TiledWorldAsset |
|--------|--------------|-----------------|
| **World data** | Owns directly | Preserved as `tiled::World` |
| **Maps** | `Vec<WorldMap>` (metadata only) | `HashMap<String, Handle<TiledMapAsset>>` (loaded) |
| **Loading** | Manual map loading required | All maps loaded as dependencies |

**Rationale:**
1. **Load all maps:** Pre-load all maps as asset dependencies
2. **Handle-based:** Maps referenced via Bevy handles
3. **Automatic tracking:** Bevy tracks all map dependencies

---

## Layer Types

### tiled::LayerData

```rust
// From tiled crate (layers/mod.rs:37)
#[derive(PartialEq, Clone, Debug)]
pub enum LayerData {
    Finite(FiniteTileLayer),
    Infinite(InfiniteTileLayer),
    Objects(ObjectLayer),
    Image(ImageLayer),
    Group(GroupLayer),
}

// Each variant contains different data
pub struct FiniteTileLayer { /* ... */ }
pub struct InfiniteTileLayer { /* ... */ }
pub struct ObjectLayer { /* ... */ }
pub struct ImageLayer { /* ... */ }
pub struct GroupLayer { /* ... */ }
```

### bevy_tiled_topdown Layer Approach

**We DON'T create asset types for layers.** Instead:

1. **Asset layer:** Layers remain as `Vec<LayerData>` inside `tiled::Map`
2. **Spawning layer:** Each layer becomes an **entity** with `TiledLayer` component
3. **Rendering layer:** Rendering systems add rendering components to layer entities

```rust
// Component for layer entities (not an asset)
#[derive(Component, Reflect)]
pub struct TiledLayer {
    pub layer_id: u32,
    pub layer_type: TiledLayerType,
}

#[derive(Reflect)]
pub enum TiledLayerType {
    Tiles,
    Objects,
    Image,
    Group,
}
```

**Rationale:**
- Layers are **parts of a map**, not independent assets
- Spawning creates entity hierarchy: Map → Layers → Objects/Tiles
- Keeps asset layer simple (just load the map)

---

## Object Types

### tiled::ObjectData

```rust
// From tiled crate (objects.rs)
pub struct ObjectData {
    pub id: u32,
    pub gid: Option<Gid>,        // Tile reference (if object is a tile)
    pub name: String,
    pub obj_type: String,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub visible: bool,
    pub shape: ObjectShape,      // Point, Rect, Ellipse, Polygon, etc.
    pub properties: Properties,
}

pub enum ObjectShape {
    Rect { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Point(f32, f32),
    Polygon { points: Vec<(f32, f32)> },
    Polyline { points: Vec<(f32, f32)> },
    Text { /* ... */ },
}
```

### bevy_tiled_topdown Object Approach

**Objects become entities with components:**

```rust
// Component for object entities (not an asset)
#[derive(Component, Reflect)]
pub enum TiledObject {
    Point,
    Rectangle { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polygon { vertices: Vec<Vec2> },
    Polyline { vertices: Vec<Vec2> },
    Tile { width: f32, height: f32 },
    Text,
}
```

**Rationale:**
- Objects are **spawned entities**, not assets
- Geometry stored in `TiledObject` component
- Custom properties become components via reflection
- Position/rotation stored in `Transform` component

---

## Tile Types

### tiled::TileData

```rust
// From tiled crate (tile.rs:33)
pub struct TileData {
    pub id: TileId,
    pub tile_type: Option<String>,
    pub properties: Properties,
    pub image: Option<Image>,           // For image collection tiles
    pub animation: Option<Vec<Frame>>,
    pub objectgroup: Option<ObjectLayer>,
    pub probability: f32,
}

// Wrapper type with lifetime (borrows from tileset)
pub struct Tile<'tileset> {
    tileset: &'tileset Tileset,
    data: &'tileset TileData,
}
```

### bevy_tiled_topdown Tile Approach

**Tiles become entities with components:**

```rust
// Component for tile entities (not an asset)
#[derive(Component, Reflect)]
pub struct TiledTile {
    pub tile_id: tiled::TileId,
    pub tileset_index: u32,
}

// Animation component (added if tile is animated)
#[derive(Component, Reflect)]
pub struct TiledAnimation {
    pub frames: Vec<TiledAnimationFrame>,
    pub current_frame: usize,
    pub elapsed: f32,
}
```

**Rationale:**
- Individual tiles are **entities** in tilemaps
- `TiledTile` references back to tileset data
- Animations added as separate component
- Tile properties extracted and added as components

---

## Summary: Ownership & Lifecycle

### tiled crate

```
Map
├─ Vec<Arc<Tileset>> ──┐
│                       │  (shared ownership via Arc)
│                       ▼
│                   Tileset (shared)
├─ Vec<LayerData>
   └─ Objects reference tilesets via GID lookup
```

- **Ownership:** Arc-based shared ownership
- **Loading:** Manual via `Loader` + `ResourceCache`
- **Lifecycle:** Reference counting via Arc
- **Dependencies:** Manually tracked in ResourceCache

### bevy_tiled_topdown

```
TiledMapAsset
├─ tiled::Map (preserved)
├─ HashMap<u32, TilesetReference>
│  └─ Handle<TiledTilesetAsset> ──┐
│                                  │  (Bevy asset handle)
├─ HashMap<String, Handle<...>>   │
│  └─ Handle<TiledTemplateAsset>  │
│                                  ▼
└─ HashMap<u32, Handle<Image>>   TiledTilesetAsset (separate asset)
                                  ├─ Arc<tiled::Tileset>
                                  ├─ Handle<Image>
                                  └─ HashMap<TileId, Handle<Image>>
```

- **Ownership:** Bevy asset handles
- **Loading:** Automatic via `AssetLoader` + `LoadContext`
- **Lifecycle:** Bevy asset system (strong/weak handles)
- **Dependencies:** Automatically tracked via `load_context.load()`

---

## Key Architectural Differences

### 1. No Lifetime Parameters

**tiled crate:** Uses wrapper types with lifetimes (`Tile<'tileset>`, `Layer<'map>`) for efficient borrowing.

**bevy_tiled_topdown:** All assets are owned or use handles. No lifetime parameters needed because:
- Assets store `Arc<tiled::T>` or owned `tiled::T`
- References use `Handle<T>` instead of `&T`
- Entities use components (copied/cloned data)

### 2. Two Representations

**tiled crate:** Single representation (parsed data).

**bevy_tiled_topdown:** Dual representation:
1. **Asset layer:** Raw `tiled::T` data + Bevy handles
2. **ECS layer:** Components on entities

### 3. Loading Strategy

**tiled crate:**
```rust
let mut loader = Loader::with_cache(cache);
let map = loader.load_tmx_map(path)?;
// Tilesets in map.tilesets() are Arc<Tileset>
```

**bevy_tiled_topdown:**
```rust
// In TiledMapAssetLoader::load()
for tileset_ref in map.tilesets() {
    let handle = load_context.load(tileset_path);
    self.tilesets.insert(index, TilesetReference { handle, first_gid });
}
// Bevy tracks dependencies automatically
```

---

## Migration Impact

When porting code from direct `tiled` crate usage:

| Old (tiled crate) | New (bevy_tiled_topdown) |
|-------------------|--------------------------|
| `map.tilesets()` → `Vec<Arc<Tileset>>` | `map_asset.tilesets` → `HashMap<u32, TilesetReference>` |
| `tileset.image` → `Option<Image>` | `tileset_asset.image` → `Option<Handle<Image>>` |
| Manual loading via `Loader` | Automatic via `asset_server.load()` |
| Arc-based sharing | Handle-based asset references |
| Parse custom properties manually | Properties pre-parsed in asset |
