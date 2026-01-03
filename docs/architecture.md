# Architecture: Three Clean Layers (Workspace)

The `bevy_tiled_topdown` **workspace** is designed with three distinct layers split across **5 separate crates**. This architecture enables clean code, testability, flexibility, and composability.

```
┌─────────────────────────────────────────────────────────────────┐
│           Layer 3: REACTIVE (Multiple Optional Crates)          │
│                                                                 │
│  bevy_tiled_render      │ Sprite-based rendering               │
│  bevy_tiled_tilemap     │ bevy_ecs_tilemap integration         │
│  bevy_tiled_avian       │ Avian physics integration            │
│  User Code              │ Custom observers, game logic         │
│                                                                 │
│  All observe core events and add specialized components        │
└─────────────────────────────────────────────────────────────────┘
                           ▲
                           │ Events (MapCreated, ObjectCreated, etc.)
                           │
┌─────────────────────────────────────────────────────────────────┐
│              Layer 2: SPAWNING (bevy_tiled_core)                │
│                                                                 │
│  Entity Creation & Property Inheritance                        │
│  • Watch for TiledMap/TiledWorld/TiledTemplate                 │
│  • Create entity hierarchies                                   │
│  • Perform property inheritance (tile→template→object)         │
│  • Populate TiledMapStorage                                    │
│  • Emit events                                                 │
└─────────────────────────────────────────────────────────────────┘
                           ▲
                           │ Assets<TiledMapAsset>, etc.
                           │
┌─────────────────────────────────────────────────────────────────┐
│            Layer 1: ASSET LOADING (bevy_tiled_assets)           │
│                                                                 │
│  Pure Data Loading                                             │
│  • TiledMapAssetLoader                                         │
│  • TiledTilesetAssetLoader                                     │
│  • TiledTemplateAssetLoader                                    │
│  • TiledWorldAssetLoader                                       │
│  • Load dependencies (tilesets, templates, images)             │
│  • Parse custom properties (NO inheritance)                    │
└─────────────────────────────────────────────────────────────────┘
                           ▲
                           │ File I/O (.tmx, .tsx, .tx, .world)
                           │
                    Tiled Files on Disk
```

## Workspace Structure

```
bevy_tiled_topdown/          # Workspace root
├── bevy_tiled_assets/       # Layer 1 crate
├── bevy_tiled_core/         # Layer 2 crate
├── bevy_tiled_render/       # Layer 3 crate (optional)
├── bevy_tiled_avian/        # Layer 3 crate (optional)
└── bevy_tiled_tilemap/      # Layer 3 crate (optional)
```

**Dependency Graph:**
```
bevy_tiled_render  ──┐
bevy_tiled_avian   ──┼──> bevy_tiled_core ──> bevy_tiled_assets ──> bevy + tiled
bevy_tiled_tilemap ──┘
```

---

## Layer 1: Asset Loading (Pure Data) → `bevy_tiled_assets` crate

**Crate:** `bevy_tiled_assets`

**Responsibility:** Load Tiled files and register all dependencies as Bevy assets.

**Principle:** NO ECS concerns. NO spawning. NO property inheritance. Just load data and track dependencies.

**Dependencies:** `bevy`, `tiled` only (no core, render, or physics)

### What This Layer Does

1. **Parse Tiled files** using the `tiled` crate
2. **Load dependencies** (tilesets, templates, images) via `load_context.load()`
3. **Store raw data** (preserve `tiled::Map`, `tiled::Tileset`, etc.)
4. **Create Bevy asset handles** for all dependencies
5. **Parse custom property VALUES** (but don't inherit or merge)
6. **Return asset structs** (`TiledMapAsset`, `TiledTilesetAsset`, etc.)

### What This Layer Does NOT Do

- ❌ Create entities
- ❌ Spawn anything
- ❌ Perform property inheritance
- ❌ Care about rendering
- ❌ Interact with ECS world
- ❌ Emit events

### Example: TiledMapAssetLoader

```rust
impl AssetLoader for TiledMapAssetLoader {
    type Asset = TiledMapAsset;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // 1. Parse the TMX file
        let mut loader = tiled::Loader::with_cache(self.cache.0.clone());
        let map = loader.load_tmx_map(path)?;

        // 2. Load tileset dependencies
        let mut tilesets = HashMap::new();
        for (index, tileset) in map.tilesets().enumerate() {
            let tileset_path = get_tileset_path(&tileset.source);
            let handle = load_context.load(tileset_path);  // ← Bevy tracks this
            tilesets.insert(index as u32, TilesetReference {
                handle,
                first_gid: tileset.first_gid,
            });
        }

        // 3. Load template dependencies
        let mut templates = HashMap::new();
        for object in map.objects() {
            if let Some(template_path) = object.template {
                let handle = load_context.load(template_path);  // ← Bevy tracks this
                templates.insert(template_path.to_string(), handle);
            }
        }

        // 4. Load image dependencies (for image layers)
        let mut images = HashMap::new();
        for layer in map.layers() {
            if let LayerData::Image(image_layer) = layer.layer_data() {
                let image_path = load_context
                    .path()
                    .parent()?
                    .join(&image_layer.image.source);
                let handle = load_context.load(image_path);  // ← Bevy tracks this
                images.insert(layer.id(), handle);
            }
        }

        // 5. Parse custom properties (but DON'T inherit)
        #[cfg(feature = "user_properties")]
        let properties = DeserializedMapProperties::parse(&map)?;

        // 6. Return the asset
        Ok(TiledMapAsset {
            map,           // ← Raw tiled data preserved
            tilesets,      // ← Bevy handles
            templates,     // ← Bevy handles
            images,        // ← Bevy handles
            properties,    // ← Parsed but not inherited
            // ... other fields
        })
    }
}
```

**Key pattern:** Use `load_context.load()` for all dependencies. Bevy automatically tracks them.

### Relative Path Handling

File-type custom properties must be converted to `Handle<T>`:

```rust
// In property parsing
match property_value {
    PropertyValue::FileValue(relative_path) => {
        // Get the asset's parent directory
        let asset_dir = load_context.path().parent()?;

        // Join relative path to parent
        let full_path = asset_dir.join(relative_path);

        // Load as dependency
        let handle: Handle<TransitionAsset> = load_context.load(full_path);

        // Store handle in properties
        // (NOT the path string)
    }
}
```

This fixes the relative path issue in `bevy_ecs_tiled`.

---

## Layer 2: Entity Spawning (ECS Creation) → `bevy_tiled_core` crate

**Crate:** `bevy_tiled_core`

**Responsibility:** Watch for `TiledMap`/`TiledWorld`/`TiledTemplate` components, create entity hierarchies, and perform property inheritance.

**Principle:** This is where assets become ECS entities. Property inheritance happens HERE, not in asset loading.

**Dependencies:** `bevy`, `bevy_tiled_assets` only (no render or physics)

### What This Layer Does

1. **Detect components** added to entities (`TiledMap`, `TiledWorld`, `TiledTemplate`)
2. **Wait for asset loading** to complete
3. **Create entity hierarchies:**
   - Map entity → Layer entities → Tilemap/Object entities → Tile entities
4. **Perform property inheritance** (tile → template → object)
5. **Insert components** on entities (geometry, custom properties, etc.)
6. **Populate `TiledMapStorage`** (bidirectional Tiled ID ↔ Entity mapping)
7. **Emit events** (`MapCreated`, `LayerCreated`, `ObjectCreated`, etc.)

### What This Layer Does NOT Do

- ❌ Load files from disk
- ❌ Parse TMX/TSX/TX files
- ❌ Add rendering components (that's Layer 3)
- ❌ Add physics components (that's Layer 3, via events)

### Example: Map Spawning System

```rust
fn spawn_map_system(
    mut commands: Commands,
    map_assets: Res<Assets<TiledMapAsset>>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    template_assets: Res<Assets<TiledTemplateAsset>>,
    q_maps: Query<(Entity, &TiledMap), Added<TiledMap>>,
) {
    for (map_entity, tiled_map) in q_maps.iter() {
        // 1. Wait for asset to load
        let Some(map_asset) = map_assets.get(&tiled_map.handle) else {
            continue;
        };

        // 2. Create storage
        let mut storage = TiledMapStorage::default();

        // 3. Spawn layers recursively
        spawn_layers(
            &mut commands,
            map_entity,
            map_asset,
            &tileset_assets,
            &template_assets,
            &mut storage,
        );

        // 4. Insert storage on map entity
        commands.entity(map_entity).insert(storage);

        // 5. Emit event
        commands.trigger_targets(
            TiledEvent::new(map_entity, MapCreated)
                .with_map(map_entity, map_asset.id()),
            map_entity,
        );
    }
}
```

### Property Inheritance Algorithm

**CRITICAL:** This happens at spawn time, NOT asset load time.

```rust
fn spawn_object_with_properties(
    commands: &mut Commands,
    object: &tiled::Object,
    tileset_asset: Option<&TiledTilesetAsset>,
    template_asset: Option<&TiledTemplateAsset>,
    map_asset: &TiledMapAsset,
    registry: &AppTypeRegistry,
) -> Entity {
    // 1. Create the entity
    let entity = commands.spawn_empty().id();

    // 2. Start with empty properties
    let mut merged = DeserializedProperties::default();

    // 3. Merge tile properties (if object uses a tile)
    if let Some(tile) = object.tile {
        if let Some(tileset) = tileset_asset {
            if let Some(tile_props) = tileset.tile_properties.get(&tile.id()) {
                merged.merge_from(tile_props);  // ← BASE LAYER
            }
        }
    }

    // 4. Merge template properties (if object uses template)
    if let Some(template) = template_asset {
        merged.merge_from(&template.properties);  // ← MIDDLE LAYER
    }

    // 5. Merge object instance properties (HIGHEST PRIORITY)
    if let Some(object_props) = map_asset.properties.objects.get(&object.id) {
        merged.merge_from(object_props);  // ← TOP LAYER (wins conflicts)
    }

    // 6. Convert merged properties to components
    let bundle = merged.to_bundle(registry);
    commands.entity(entity).insert(bundle);

    // 7. Add geometry component
    commands.entity(entity).insert(TiledObject::from(object));

    // 8. Add transform
    commands.entity(entity).insert(Transform::from_xyz(object.x, object.y, 0.0));

    entity
}
```

**Merge strategy:**
- If a property exists in multiple layers, **higher layer wins**
- Tile properties are the **base** (lowest priority)
- Template properties **override** tile properties
- Object properties **override** everything (highest priority)

### Entity Hierarchy

```
Map Entity (TiledMap)
├─ Layer Entity (TiledLayer::Tiles)
│  └─ Tilemap Entity (TiledTilemap)
│     └─ Tile Entity (TiledTile)
│        └─ [Custom property components]
│
├─ Layer Entity (TiledLayer::Objects)
│  └─ Object Entity (TiledObject::Rectangle)
│     ├─ [Custom property components] ← Inherited from tile → template → object
│     └─ Visual Entity (TiledObjectVisualOf) [for tile-type objects]
│
├─ Layer Entity (TiledLayer::Image)
│  └─ Image Entity (Sprite)
│
└─ Layer Entity (TiledLayer::Group)
   └─ [Nested layers...] ← Recursive!
```

---

## Layer 3: Reactive Systems (User Logic) → Multiple optional crates

**Crates:** `bevy_tiled_render`, `bevy_tiled_avian`, `bevy_tiled_tilemap`, user code

**Responsibility:** React to events and add custom behavior (rendering, physics, game logic).

**Principle:** Optional, composable plugins. Users pick and choose. All are equal peers that observe core events.

**Dependencies:** Each depends on `bevy` + `bevy_tiled_core` + their specialized deps (avian2d, bevy_ecs_tilemap, etc.)

### What This Layer Does

1. **Observe events** (`On<TiledEvent<ObjectCreated>>`, etc.)
2. **Add rendering components** (optional, separate plugin)
3. **Add physics colliders** (via physics plugin)
4. **Add game-specific logic** (triggers, NPCs, etc.)
5. **Query entities** using `TiledMapStorage`

### What This Layer Does NOT Do

- ❌ Load assets
- ❌ Create the core entity hierarchy (that's Layer 2)
- ❌ Parse Tiled files

### Example: Rendering Plugin → `bevy_tiled_render` crate

**File:** `bevy_tiled_render/src/tile.rs`

```rust
// bevy_tiled_render crate
use bevy::prelude::*;
use bevy_tiled_core::prelude::*;
use bevy_tiled_assets::prelude::*;

pub struct TiledRenderPlugin;

impl Plugin for TiledRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tile_created);
        app.add_observer(on_object_created);
    }
}

fn on_tile_created(
    trigger: On<TiledEvent<TileCreated>>,
    mut commands: Commands,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    q_tiles: Query<&TiledTile>,
) {
    // Get tile data from core
    let Ok(tile) = q_tiles.get(trigger.entity) else { return };

    // Get tileset asset
    let Some(tileset_asset) = trigger.get_tileset_asset(&tileset_assets) else { return };

    // Add sprite rendering (bevy_tiled_render's job)
    commands.entity(trigger.entity).insert((
        Sprite::from_atlas_image(/* ... */),
        TextureAtlas::from(/* ... */),
    ));
}
```

### Example: Physics Integration → `bevy_tiled_avian` crate

**File:** `bevy_tiled_avian/src/collider.rs`

```rust
// bevy_tiled_avian crate
use bevy::prelude::*;
use bevy_tiled_core::prelude::*;
use avian2d::prelude::*;

pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(PhantomData<T>);

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_observer(on_layer_created::<T>);
        app.add_observer(on_object_created::<T>);
    }
}

fn on_layer_created<T: TiledPhysicsBackend>(
    trigger: On<TiledEvent<LayerCreated>>,
    mut commands: Commands,
    // Extract geometry from tiles, generate colliders
) {
    // bevy_tiled_avian's job: observe core events, add physics
    let geometry = extract_tile_geometry(/* ... */);
    let colliders = backend.spawn_colliders(&mut commands, &trigger, &geometry);
}
```

### Example: Game Logic

```rust
#[derive(Component, Reflect)]
struct Door {
    target_map: String,
    locked: bool,
}

fn on_door_created(
    trigger: On<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    q_doors: Query<&Door>,
) {
    let Ok(door) = q_doors.get(trigger.entity) else { return };

    // Add game-specific components
    commands.entity(trigger.entity).insert((
        InteractionTrigger,
        Sensor,
        Collider::rectangle(32.0, 32.0),
    ));

    if door.locked {
        commands.entity(trigger.entity).insert(LockedIcon);
    }
}

fn setup(app: &mut App) {
    app.add_observer(on_door_created);
}
```

---

## Data Flow Through Layers

### Loading a Map

```
1. User spawns entity with TiledMap component
   commands.spawn(TiledMap { handle: asset_server.load("map.tmx") });

2. LAYER 1: Asset Loading
   ├─ TiledMapAssetLoader::load() called by Bevy
   ├─ Parses map.tmx using tiled crate
   ├─ Calls load_context.load() for each tileset
   ├─ Calls load_context.load() for each template
   ├─ Calls load_context.load() for each image
   ├─ Parses custom properties (no inheritance)
   └─ Returns TiledMapAsset

3. LAYER 2: Entity Spawning
   ├─ spawn_map_system detects Added<TiledMap>
   ├─ Waits for Assets<TiledMapAsset> to be ready
   ├─ Creates entity hierarchy (layers, objects, tiles)
   ├─ For each object:
   │  ├─ Loads tileset asset (if object uses tile)
   │  ├─ Loads template asset (if object uses template)
   │  ├─ Merges properties: tile → template → object
   │  └─ Inserts components on entity
   ├─ Populates TiledMapStorage
   └─ Emits TiledEvent<MapCreated>

4. LAYER 3: Reactive Systems
   ├─ User observers receive TiledEvent<ObjectCreated>
   ├─ Rendering plugin adds rendering components
   ├─ Physics plugin adds colliders
   └─ Game logic adds custom behavior
```

### Property Inheritance Flow

```
LAYER 1 (Asset Loading):
  TiledTilesetAsset
  ├─ tile_properties: HashMap<TileId, DeserializedProperties>
  │  └─ tile_id=42: { health: 100, damage: 10 }

  TiledTemplateAsset
  ├─ properties: DeserializedProperties
  │  └─ { damage: 20, armor: 5 }  ← Overrides tile's damage

  TiledMapAsset
  ├─ properties.objects: HashMap<u32, DeserializedProperties>
     └─ object_id=7: { armor: 10 }  ← Overrides template's armor

LAYER 2 (Entity Spawning):
  spawn_object_with_properties(object_id=7)

  Step 1: Start empty
    merged = {}

  Step 2: Merge tile properties (object uses tile_id=42)
    merged = { health: 100, damage: 10 }

  Step 3: Merge template properties
    merged = { health: 100, damage: 20, armor: 5 }  ← damage overridden

  Step 4: Merge object properties
    merged = { health: 100, damage: 20, armor: 10 }  ← armor overridden

  Step 5: Convert to components
    entity.insert((
      Health(100),
      Damage(20),
      Armor(10),
    ))

LAYER 3 (Reactive):
  on_object_created observer
  ├─ Query<(&Health, &Damage, &Armor)>
  └─ Can now use the merged properties
```

---

## Benefits of This Architecture

### 1. Clean Separation of Concerns

Each layer has ONE job:
- **Layer 1:** Load data
- **Layer 2:** Create entities
- **Layer 3:** Add behavior

### 2. Testability

Each layer can be tested independently:
- **Layer 1:** Unit test asset loaders (input TMX → output asset)
- **Layer 2:** Unit test spawning (input asset → output entity hierarchy)
- **Layer 3:** Unit test observers (input event → output components)

### 3. Flexibility

Users can:
- Replace Layer 3 entirely (custom rendering, custom physics)
- Extend Layer 2 via observers
- Layer 1 is fixed (asset loading)

### 4. Rendering-Agnostic

Core crate (Layers 1 & 2) has NO rendering dependency:
- No `bevy_ecs_tilemap`
- No custom rendering
- Rendering is added in Layer 3 via plugins or observers

### 5. Property Inheritance at the Right Layer

- **Layer 1:** Stores raw property values (no merging)
- **Layer 2:** Performs inheritance during spawning (clean, flexible)
- **Layer 3:** Receives final merged properties on entities

### 6. Asset Lifecycle

Bevy's asset system handles:
- Loading
- Hot reloading
- Unloading
- Dependency tracking

No manual resource management needed.

---

## Comparison with bevy_ecs_tiled

### bevy_ecs_tiled Architecture

```
Single Crate (Mixed Concerns)
├─ Asset loading + entity spawning together
├─ Property handling during asset load
├─ Tightly coupled to bevy_ecs_tilemap
├─ Physics feature-gated in same crate
└─ Rendering is part of core
```

### bevy_tiled_topdown Workspace Architecture

```
bevy_tiled_assets     → Layer 1: Asset Loading (pure data)
bevy_tiled_core       → Layer 2: Entity Spawning (ECS creation)
bevy_tiled_render     → Layer 3: Sprite rendering (optional)
bevy_tiled_tilemap    → Layer 3: Tilemap rendering (optional)
bevy_tiled_avian      → Layer 3: Physics (optional)
User code             → Layer 3: Game logic
```

**Key differences:**
1. **Separation:** Five crates with clear boundaries vs. one monolithic crate
2. **Property inheritance:** Spawn-time vs. asset-load-time
3. **Rendering:** Optional separate crates vs. built-in
4. **Physics:** Optional separate crate vs. feature-gated
5. **Testability:** Each crate testable independently vs. monolithic
6. **Flexibility:** Users compose exactly what they need vs. all-or-nothing
7. **Dependencies:** Minimal deps per crate vs. everything together
8. **Composability:** Mix render+physics, tilemap+physics, or any combination
