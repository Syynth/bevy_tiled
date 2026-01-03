# bevy_tiled_topdown Documentation

Design documentation for the `bevy_tiled_topdown` crate - a rewrite of `bevy_ecs_tiled` with first-class asset support and clean architectural layering.

---

## Quick Links

- **[Architecture](architecture.md)** - The three-layer architecture (Asset Loading → Entity Spawning → Reactive Systems)
- **[Type Mapping](type-mapping.md)** - Detailed comparison of `tiled` crate types vs `bevy_tiled_topdown` asset types
- **[Design Decisions](design-decisions.md)** - Key design decisions and their rationale
- **[Property System](property-system.md)** - Custom properties and inheritance system
- **[Physics Integration](physics-integration.md)** - Avian physics backend integration (Rapier dropped)
- **[Components Comparison](components-comparison.md)** - Component/resource comparison with bevy_ecs_tiled

---

## Overview

`bevy_tiled_topdown` is designed around **three clean layers**:

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: REACTIVE (User Code, Plugins)                 │
└─────────────────────────────────────────────────────────┘
                           ▲
                    Events (MapCreated, etc.)
                           │
┌─────────────────────────────────────────────────────────┐
│  Layer 2: SPAWNING (Entity Creation, Property Inherit)  │
└─────────────────────────────────────────────────────────┘
                           ▲
                  Assets (TiledMapAsset, etc.)
                           │
┌─────────────────────────────────────────────────────────┐
│  Layer 1: ASSET LOADING (Pure Data)                     │
└─────────────────────────────────────────────────────────┘
                           ▲
                  Tiled Files (.tmx, .tsx, etc.)
```

**Read more:** [architecture.md](architecture.md)

---

## Key Improvements Over bevy_ecs_tiled

### 1. First-Class Asset Support

**Problem:** `bevy_ecs_tiled` embeds tilesets in maps, doesn't support templates as standalone assets.

**Solution:** All Tiled file types are separate Bevy assets:
- `TiledTilesetAsset` (`.tsx`) - Separate loadable asset
- `TiledTemplateAsset` (`.tx`) - Separate loadable asset, can spawn standalone
- `TiledMapAsset` (`.tmx`) - References tilesets/templates via `Handle<T>`
- `TiledWorldAsset` (`.world`) - References maps via `Handle<T>`

**Benefits:**
- Asset sharing (multiple maps use one tileset)
- Hot reloading (edit tileset, all maps update)
- Template spawning (`TiledTemplate` component)

**Read more:** [design-decisions.md#decision-3-first-class-asset-support](design-decisions.md#decision-3-first-class-asset-support-for-all-tiled-files)

---

### 2. Property Inheritance at Spawn Time

**Problem:** `bevy_ecs_tiled` doesn't support property inheritance from tile → template → object.

**Solution:** Properties stored raw in assets, merged during entity spawning:

```rust
// Asset layer (NO merging)
TiledTilesetAsset { tile_properties: { 42: { health: 100 } } }
TiledTemplateAsset { properties: { damage: 20 } }
TiledMapAsset { properties.objects: { 7: { armor: 10 } } }

// Spawning layer (MERGES)
Entity {
    Health(100),   // from tile
    Damage(20),    // from template
    Armor(10),     // from object
}
```

**Merge order:** Tile → Template → Object (object wins conflicts)

**Read more:** [property-system.md#property-inheritance-spawn-time](property-system.md#property-inheritance-spawn-time)

---

### 3. Relative Path Support (File Properties)

**Problem:** `bevy_ecs_tiled` doesn't properly handle file-type custom properties with relative paths.

**Solution:** Convert relative paths to `Handle<T>` during asset loading:

```rust
// Tiled property: "../transitions/fade.transition.toml"

// Asset loading:
let asset_dir = load_context.path().parent();
let handle: Handle<TransitionAsset> = load_context.load(asset_dir.join(relative_path));

// Component on entity:
TransitionReference {
    transition: Handle<TransitionAsset> { /* ... */ }
}
```

**Read more:** [property-system.md#file-properties-handles](property-system.md#file-properties-handles)

---

### 4. Rendering-Agnostic Core

**Problem:** `bevy_ecs_tiled` is tightly coupled to `bevy_ecs_tilemap`.

**Solution:** Core has NO rendering dependency. Rendering added via:
- Separate rendering plugin (e.g., `bevy_tiled_topdown_tilemap`)
- User observers on `TileCreated` events
- Custom rendering implementations

**Benefits:**
- Use any rendering approach
- Smaller core dependency tree
- Can adopt new rendering tech without breaking core

**Read more:** [design-decisions.md#decision-1-rendering-agnostic-core](design-decisions.md#decision-1-rendering-agnostic-core)

---

### 5. Template Spawning

**Problem:** `bevy_ecs_tiled` doesn't support spawning templates as standalone entities.

**Solution:** New `TiledTemplate` component:

```rust
// Spawn template without a map
commands.spawn(TiledTemplate {
    handle: asset_server.load("templates/chest.tx"),
    position: Some(Vec2::new(100.0, 200.0)),
    rotation: None,
});
```

**Use cases:**
- Spawn NPCs/enemies procedurally
- Place objects at runtime
- Template as prefab system

**Read more:** [type-mapping.md#template-types](type-mapping.md#template-types)

---

### 6. Clean Layer Separation

**Problem:** `bevy_ecs_tiled` mixes asset loading, entity spawning, and rendering.

**Solution:** Three distinct layers:

**Layer 1: Asset Loading** (pure data)
- Load `.tmx`/`.tsx`/`.tx` files
- Register dependencies
- Parse property VALUES
- NO ECS, NO spawning

**Layer 2: Entity Spawning** (ECS creation)
- Detect `TiledMap`/`TiledWorld`/`TiledTemplate` components
- Create entity hierarchy
- Perform property inheritance
- Emit events

**Layer 3: Reactive Systems** (user logic)
- Observers react to events
- Add rendering components
- Add physics colliders
- Add game logic

**Read more:** [architecture.md](architecture.md)

---

## Type Mapping Reference

Quick reference for how `tiled` crate types map to our asset types:

| tiled crate | bevy_tiled_topdown |
|-------------|-------------------|
| `tiled::Map` | `TiledMapAsset { map: tiled::Map, tilesets: HashMap<u32, Handle<TiledTilesetAsset>>, ... }` |
| `tiled::Tileset` | `TiledTilesetAsset { tileset: Arc<tiled::Tileset>, image: Handle<Image>, ... }` |
| `tiled::Template` | `TiledTemplateAsset { template: tiled::Template, tileset: Handle<TiledTilesetAsset>, ... }` |
| `tiled::World` | `TiledWorldAsset { world: tiled::World, maps: HashMap<String, Handle<TiledMapAsset>> }` |
| `Vec<Arc<Tileset>>` | `HashMap<u32, TilesetReference { handle: Handle<...>, first_gid }>` |
| `Arc<T>` sharing | `Handle<T>` asset references |
| `Option<Image>` (path) | `Option<Handle<Image>>` (loaded texture) |

**Full details:** [type-mapping.md](type-mapping.md)

---

## Design Principles

1. **Preserve raw data** - Keep `tiled::Map`, `tiled::Tileset`, etc. intact in assets
2. **Handle-based references** - Use `Handle<T>` instead of `Arc<T>`
3. **Clean layering** - Asset loading → Entity spawning → Reactive logic
4. **Property inheritance at spawn** - Assets store raw values, spawning merges them
5. **Rendering-agnostic** - Core has no rendering dependency
6. **First-class everything** - All Tiled file types are Bevy assets
7. **Event-driven** - Observers for extensibility
8. **Bevy-native** - Uses standard Bevy patterns (assets, handles, components, events)

**Full details:** [design-decisions.md](design-decisions.md)

---

## Property System Summary

```
1. Define Rust types with #[derive(Reflect)]
2. Register types in Bevy app
3. Export types to JSON for Tiled editor
4. Import JSON in Tiled, use custom properties
5. Asset loading parses property VALUES
6. Entity spawning performs inheritance (tile → template → object)
7. Components inserted on entities
8. Query/react using standard Bevy ECS
```

**Full details:** [property-system.md](property-system.md)

---

## Migration from bevy_ecs_tiled

### Breaking Changes

1. **Imports:** `bevy_ecs_tiled` → `bevy_tiled_topdown`
2. **Rendering:** Core doesn't include rendering - add plugin or observers
3. **Asset structure:** Maps contain `Handle<TiledTilesetAsset>` instead of embedded tilesets
4. **Property inheritance:** Now explicit at spawn time (transparent to users)

### Preserved API

- `TiledMap`, `TiledWorld` components (same usage)
- `TiledObject`, `TiledLayer` components (mostly same)
- Custom properties system (identical API)
- Physics backend trait (same interface)
- Event system (same pattern)
- Storage API (`TiledMapStorage` query methods)

### New Features

- `TiledTemplate` component
- Property inheritance (tile → template → object)
- File properties with relative paths work correctly
- Hot reload for tilesets/templates
- Rendering flexibility

---

## Document Index

### Core Concepts

- **[Architecture](architecture.md)** - Three-layer architecture, data flow, entity hierarchy
- **[Type Mapping](type-mapping.md)** - tiled crate types vs asset types, ownership models
- **[Design Decisions](design-decisions.md)** - Key decisions and rationale
- **[Property System](property-system.md)** - Custom properties, inheritance, file properties
- **[Physics Integration](physics-integration.md)** - Avian backend, collider generation, event-driven physics
- **[Components Comparison](components-comparison.md)** - Complete comparison with bevy_ecs_tiled

### Quick References

**Asset types:**
- [TiledMapAsset](type-mapping.md#tiledmapasset-our-design)
- [TiledTilesetAsset](type-mapping.md#tiledtilesetasset-our-design)
- [TiledTemplateAsset](type-mapping.md#tiledtemplateasset-our-design)
- [TiledWorldAsset](type-mapping.md#tiledworldasset-our-design)

**Architecture layers:**
- [Layer 1: Asset Loading](architecture.md#layer-1-asset-loading-pure-data)
- [Layer 2: Entity Spawning](architecture.md#layer-2-entity-spawning-ecs-creation)
- [Layer 3: Reactive Systems](architecture.md#layer-3-reactive-systems-user-logic)

**Key features:**
- [Property Inheritance Algorithm](property-system.md#property-inheritance-spawn-time)
- [File Property Handling](property-system.md#file-properties-handles)
- [Entity References](property-system.md#entity-references-object-properties)
- [Rendering Integration](architecture.md#example-rendering-plugin-optional)

---

## Questions & Answers

### Why preserve tiled::Map instead of custom types?

**Answer:** Gives users full access to all Tiled data without re-implementing the tiled crate. Future-proof against new tiled crate features.

**Details:** [design-decisions.md#decision-4-preserve-raw-tiled-crate-types](design-decisions.md#decision-4-preserve-raw-tiled-crate-types-in-assets)

---

### Why not merge properties during asset loading?

**Answer:** Assets should be pure data. Merging is logic that belongs in the spawning layer. Enables debugging, hot reload, and flexible merge rules.

**Details:** [design-decisions.md#decision-2-property-inheritance-at-spawn-time](design-decisions.md#decision-2-property-inheritance-at-spawn-time)

---

### How do I add rendering?

**Answer:** Either use a rendering plugin (when available) or add observers that react to `TiledEvent<TileCreated>` and insert rendering components.

**Details:** [architecture.md#example-rendering-plugin-optional](architecture.md#example-rendering-plugin-optional)

---

### How does property inheritance work?

**Answer:** During entity spawning, properties are merged from tile → template → object, with object properties winning conflicts. The merged result becomes components on the entity.

**Details:** [property-system.md#property-inheritance-spawn-time](property-system.md#property-inheritance-spawn-time)

---

### Can I spawn templates without a map?

**Answer:** Yes! Use `TiledTemplate` component:

```rust
commands.spawn(TiledTemplate {
    handle: asset_server.load("template.tx"),
    position: Some(Vec2::new(100.0, 200.0)),
    rotation: None,
});
```

**Details:** [type-mapping.md#tiledtemplateasset-our-design](type-mapping.md#tiledtemplateasset-our-design)

---

## Contributing

When contributing to this design:

1. **Update relevant docs** when making architectural changes
2. **Keep examples simple** and focused on one concept
3. **Cross-reference** related sections using markdown links
4. **Test examples** to ensure they compile and work as described

---

## Implementation Status

This is currently **design documentation** for a planned implementation. Refer to `../notes.md` and the plan file for implementation roadmap.
