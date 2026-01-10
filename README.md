# bevy_tiled

[Tiled](https://www.mapeditor.org/) map integration for [Bevy](https://bevyengine.org/).

| Bevy | bevy_tiled |
|------|------------|
| 0.17 | main       |

## Quick Start

```rust
use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyTiledmapPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMap {
        handle: asset_server.load("map.tmx"),
    });
}
```

## Examples

### Loading a World

Load multiple connected maps from a `.world` file:

```rust
commands.spawn(TiledWorld {
    handle: asset_server.load("overworld.world"),
});
```

### Custom Configuration

```rust
App::new()
    .add_plugins(
        BevyTiledmapPlugin::default()
            .with_core(TiledmapCoreConfig {
                // Export Rust types to Tiled for autocomplete
                export_target: Some(TypeExportTarget::TiledProject),
                project_path: Some("my_game.tiled-project".into()),
                ..default()
            })
            .with_tilemap(TilemapRenderConfig {
                enable_animations: true,
                enable_parallax: true,
                ..default()
            }),
    )
```

### Custom Components

Define Rust components that map directly to Tiled custom types:

```rust
#[derive(Component, Reflect, TiledClass)]
#[tiled(name = "Enemy")]
#[reflect(Component)]
struct Enemy {
    #[tiled(default = 100)]
    health: i32,

    #[tiled(default = 5.0)]
    speed: f32,

    patrol_path: Option<String>,
}

// Objects with type "Enemy" in Tiled automatically get this component
fn on_enemy_spawn(query: Query<(&Enemy, &Transform), Added<Enemy>>) {
    for (enemy, transform) in &query {
        info!("Enemy spawned at {:?} with {} health",
              transform.translation, enemy.health);
    }
}
```

### Physics with Avian2D

```rust
use avian2d::prelude::*;
use bevy_tiledmap_avian::prelude::*;

App::new()
    .add_plugins(PhysicsPlugins::default())
    .add_plugins(TiledmapAssetsPlugin)
    .add_plugins(TiledmapCorePlugin::default())
    .add_plugins(TiledmapAvianPlugin::default())
```

Object shapes (rectangles, ellipses, polygons) automatically become physics colliders.

### Infinite Maps

Tiled's infinite/chunk-based maps work out of the box:

```rust
commands.spawn(TiledMap {
    handle: asset_server.load("infinite_dungeon.tmx"),
});
```

### Running Examples

```bash
cargo run --example quick_start
cargo run --example demo
cargo run --example infinite_map
cargo run --example custom_config
```

## Credits

This project is heavily inspired by [bevy_ecs_tiled](https://github.com/adrien-music/bevy_ecs_tiled). Thank you to the maintainers for their excellent work on Tiled integration for Bevy.

## Disclaimer

This library is **experimental**, **probably broken**, and **not ready for production use**. The API will change without warning. Use at your own risk.
