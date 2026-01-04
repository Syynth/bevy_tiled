# bevy_tiledmap

[![Crates.io](https://img.shields.io/crates/v/bevy_tiledmap.svg)](https://crates.io/crates/bevy_tiledmap)
[![Docs](https://docs.rs/bevy_tiledmap/badge.svg)](https://docs.rs/bevy_tiledmap)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE)

Tiled map loader and integration for Bevy.

## Quick Start

```rust
use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyTiledmapPlugin::default())
        .add_systems(Startup, spawn_map)
        .run();
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TiledMap {
        handle: asset_server.load("map.tmx"),
    });
}
```

That's it! Your Tiled map will be loaded, spawned as entities, and rendered.

## Features

This crate provides optional feature flags for different integrations:

- **`tilemap`** (default): High-performance tile rendering using `bevy_ecs_tilemap`
- **`avian`**: Physics collider generation using `avian2d`
- **`native`**: Future support for Bevy's native tilemap rendering (placeholder)

### Feature Flag Examples

```toml
# Default (includes tilemap rendering)
[dependencies]
bevy_tiledmap = "0.1"

# Core only (no rendering, no physics)
[dependencies]
bevy_tiledmap = { version = "0.1", default-features = false }

# With physics (tilemap + avian)
[dependencies]
bevy_tiledmap = { version = "0.1", features = ["avian"] }

# All features
[dependencies]
bevy_tiledmap = { version = "0.1", features = ["avian", "native"] }
```

## What This Crate Does

`bevy_tiledmap` is a unified meta-crate that combines all `bevy_tiledmap_*` sub-crates:

### Layer 1: Asset Loading
- Loads Tiled files (`.tmx`, `.tsx`, `.tx`, `.world`) as Bevy assets
- Parses all Tiled features: maps, tilesets, objects, properties, animations, etc.

### Layer 2: Entity Spawning
- Converts loaded assets into structured ECS hierarchies
- Creates entities for maps, layers, and objects
- Merges properties from maps, layers, templates, and tiles
- Triggers events for Layer 3 plugins to observe

### Layer 3: Integration Plugins (Optional)
- **Tilemap Rendering** (`tilemap` feature): Batched tile rendering with `bevy_ecs_tilemap`
- **Physics** (`avian` feature): Automatic collider generation from Tiled objects and tiles
- **Native Rendering** (`native` feature): Future support for Bevy's built-in tilemap (not yet implemented)

## Custom Configuration

You can customize the behavior of each layer:

```rust
use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(
        BevyTiledmapPlugin::default()
            .with_core(TiledmapCoreConfig {
                export_types_path: Some("assets/tiled_types.json".into()),
            })
            .with_tilemap(TilemapRenderConfig {
                enable_animations: true,
                enable_parallax: true,
                ..default()
            })
            .with_avian(PhysicsConfig {
                default_friction: 0.3,
                enable_tile_colliders: true,
                ..default()
            })
    )
    .run();
```

## Using Individual Crates

You can also use the individual sub-crates directly if you prefer more control:

```rust
use bevy::prelude::*;
use bevy_tiledmap_assets::TiledmapAssetsPlugin;
use bevy_tiledmap_core::prelude::*;
use bevy_tiledmap_tilemap::TilemapPlugin;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(TiledmapAssetsPlugin)
    .add_plugins(TiledmapCorePlugin::default())
    .add_plugins(TilemapPlugin::default())
    .run();
```

## Architecture

`bevy_tiledmap` follows a 3-layer architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: Integration Plugins (Optional, Feature-Gated)        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Tilemap    │  │    Avian     │  │    Native    │         │
│  │  Rendering   │  │   Physics    │  │  (Placeholder)│        │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Observes events)
┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: Entity Spawning (Core)                               │
│  • Spawns maps, layers, objects as entities                   │
│  • Property merging & inheritance                             │
│  • Triggers events for Layer 3                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Depends on)
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: Asset Loading                                        │
│  • Loads .tmx, .tsx, .tx, .world files                        │
│  • Registers Tiled asset types                                │
└─────────────────────────────────────────────────────────────────┘
```

## Sub-Crates

This meta-crate re-exports the following sub-crates:

- [`bevy_tiledmap_assets`](../bevy_tiledmap_assets): Asset loading (Layer 1)
- [`bevy_tiledmap_core`](../bevy_tiledmap_core): Entity spawning (Layer 2)
- [`bevy_tiledmap_tilemap`](../bevy_tiledmap_tilemap): Tilemap rendering (Layer 3, `tilemap` feature)
- [`bevy_tiledmap_avian`](../bevy_tiledmap_avian): Avian2D physics (Layer 3, `avian` feature)
- [`bevy_tiledmap_native`](../bevy_tiledmap_native): Native rendering placeholder (Layer 3, `native` feature)
- [`bevy_tiledmap_macros`](../bevy_tiledmap_macros): Proc-macro support

## Examples

Check out the `examples/` directory for complete working examples:

- `quick_start.rs`: Basic usage
- `with_physics.rs`: Using Avian physics integration
- `custom_config.rs`: Custom configuration patterns

## Compatibility

| bevy_tiledmap | Bevy | Tiled |
|--------------|------|-------|
| 0.1.x        | 0.17 | 1.11+ |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
