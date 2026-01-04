# bevy_tiledmap_tilemap

High-performance tile layer rendering for `bevy_tiled` using `bevy_ecs_tilemap`.

This is a **Layer 3 plugin** that observes spawning events from `bevy_tiledmap_core` and adds rendering components using `bevy_ecs_tilemap` for optimal batched tile rendering.

## Features

- ✅ **Tile layers** - Batched rendering with `bevy_ecs_tilemap`
- ✅ **Multi-tileset support** - Handles layers using multiple tilesets seamlessly
- ✅ **Tile animations** - Automatic frame cycling based on Tiled's animation data
- ✅ **Object rendering** - Sprites for tile objects, debug gizmos for shapes
- ✅ **Image layers** - Simple sprite rendering for background/foreground images
- ✅ **Parallax scrolling** - Layer parallax based on Tiled `parallaxX`/`parallaxY` properties
- ✅ **Z-ordering** - Automatic depth sorting for layers and objects
- ✅ **Flip flags** - Correct rendering of flipped tiles (horizontal, vertical, diagonal)

## Quick Start

```rust
use bevy::prelude::*;
use bevy_tiledmap_assets::BevyTiledAssetsPlugin;
use bevy_tiledmap_core::prelude::*;
use bevy_tiledmap_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(BevyTiledAssetsPlugin)
        .add_plugins(BevyTiledCorePlugin::default())
        .add_plugins(BevyTiledTilemapPlugin::default())
        .add_systems(Startup, spawn_map)
        .run();
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMap {
        handle: asset_server.load("maps/level1.tmx"),
    });
}
```

## Examples

Run examples from the workspace root:

```bash
# Basic tilemap rendering
cargo run --example basic_tilemap

# Animated tiles (water, lava, etc.)
cargo run --example animated_tiles --features animations

# Parallax scrolling
cargo run --example parallax_layers --features parallax

# All features combined
cargo run --example all_layers --features animations,parallax,debug_shapes
```

**Note**: Examples expect `.tmx` map files in an `assets/maps/` directory. Create maps in [Tiled](https://www.mapeditor.org/) and place them in your project's assets folder.

## Features

### Default

- `animations` - Tile animation support
- `parallax` - Parallax scrolling

### Optional

- `debug_shapes` - Gizmo rendering for object shapes (rectangles, ellipses, polygons, etc.)

## Tile Animations

Animations are automatically extracted from your tileset's animation data. Control them at runtime:

```rust
use bevy_tiledmap_tilemap::prelude::*;

fn control_animations(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut speed: ResMut<AnimationSpeed>,
) {
    // Pause/resume
    if keyboard.just_pressed(KeyCode::Space) {
        commands.insert_resource(AnimationsPaused);
    }

    // Adjust speed
    speed.0 = 2.0; // 2x speed
}
```

## Parallax Scrolling

Set custom properties on layers in Tiled:
- `parallaxX` (float) - Horizontal parallax factor (default: 1.0)
- `parallaxY` (float) - Vertical parallax factor (default: 1.0)

Lower values make layers appear further away (move slower).

```rust
// Mark your camera for parallax
commands.spawn((Camera2d, ParallaxCamera));
```

## Z-Ordering

Layers are automatically depth-sorted based on their order in Tiled. Customize via `ZOrderConfig`:

```rust
app.insert_resource(ZOrderConfig {
    layer_separation: 10.0,  // Z-space between layers
    object_z_offset: 1.0,    // Objects render above their layer
});
```

## Debug Shapes

Enable the `debug_shapes` feature to see gizmo outlines for all object shapes:

```rust
app.insert_resource(TilemapRenderConfig {
    enable_debug_shapes: true,
    ..default()
});
```

## Architecture

This crate follows the `bevy_tiled` three-layer architecture:

- **Layer 1** (`bevy_tiledmap_assets`) - Loads and parses Tiled files
- **Layer 2** (`bevy_tiledmap_core`) - Spawns entities and emits events
- **Layer 3** (`bevy_tiledmap_tilemap`) - Observes events and adds rendering ← **This crate**

Layer 3 plugins are:
- **Independent** - No modification of Layer 2 required
- **Composable** - Multiple rendering backends can coexist
- **Event-driven** - React to spawning events via Bevy observers
- **Data-rich** - Use pre-processed data from Layer 2

## License

MIT OR Apache-2.0
