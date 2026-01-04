# bevy_tiled_avian

Avian2D physics integration for `bevy_tiled` - automatic collider generation from Tiled maps.

## Features

- **🎯 Object Colliders**: Generate physics colliders from all Tiled object shapes (Rectangle, Ellipse, Polygon, Polyline, Point, Tile)
- **🧱 Tile Colliders**: Automatic collider generation from tileset collision shapes with intelligent rectangle merging (5-100x reduction)
- **⚙️ Property-Based Configuration**: Configure physics parameters directly in Tiled using `PhysicsSettings` custom class
- **🎭 Collision Filtering**: User-defined collision groups and masks for precise interaction control
- **🚀 Multiple Strategies**: Choose between `PerTileEntity`, `CompoundMerged`, or `CompoundChunked` for tile colliders
- **📝 Type Safety**: Full type export to Tiled for autocomplete and validation

## Quick Start

```toml
# Cargo.toml
[dependencies]
bevy = "0.17"
avian2d = "0.4"
bevy_tiled_avian = "0.1"
bevy_tiled_assets = "0.1"
bevy_tiled_core = "0.1"
```

```rust
use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_tiled_avian::prelude::*;
use bevy_tiled_assets::prelude::*;
use bevy_tiled_core::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default()) // Optional: visualize colliders
        .add_plugins(BevyTiledAssetsPlugin)
        .add_plugins(BevyTiledCorePlugin::default())
        .add_plugins(BevyTiledAvianPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        TiledMap {
            handle: asset_server.load("maps/level1.tmx"),
        },
    ));
}
```

## Configuration

### Global Physics Configuration

Customize default physics parameters via `PhysicsConfig`:

```rust
App::new()
    .add_plugins(BevyTiledAvianPlugin::new(
        PhysicsConfig {
            default_friction: 0.3,
            default_restitution: 0.1,
            default_body_type: RigidBody::Static,
            enable_tile_colliders: true,
            tile_collider_strategy: TileColliderStrategy::CompoundMerged,
            ..default()
        }
    ))
    .run();
```

### Property-Based Physics Configuration

Configure individual objects in Tiled using the `PhysicsSettings` custom class.

#### 1. Export Types (One-Time Setup)

Run the type export example to generate `physics-types.json`:

```bash
cargo run --example type_export
```

#### 2. Import Types in Tiled

1. Open Tiled
2. View → Custom Types Editor
3. Click "Import" and select `physics-types.json`

#### 3. Configure Objects in Tiled

1. Select an object in your map
2. In Properties panel, click "+" to add a property
3. Name: `physics_settings`
4. Type: `avian::PhysicsSettings`
5. Configure nested properties:

```yaml
physics_settings: avian::PhysicsSettings
  body_type: "Dynamic"      # Static, Dynamic, or Kinematic
  friction: 0.8             # 0.0 (ice) to 1.0+ (sticky)
  restitution: 0.3          # 0.0 (no bounce) to 1.0 (perfect bounce)
  density: 2.0              # Mass per unit area (kg/m²)
  is_sensor: false          # true for triggers
  lock_rotation: false      # true to prevent rotation
  linear_damping: 0.5       # Optional: reduces linear velocity
  angular_damping: 0.3      # Optional: reduces angular velocity
  gravity_scale: 1.0        # Optional: gravity multiplier
  collision_groups: "player"         # Optional: group memberships
  collision_mask: "ground,enemies"   # Optional: collision filters
```

#### Example Configurations

**Bouncy Ball** (Dynamic):
```yaml
body_type: "Dynamic"
friction: 0.1
restitution: 0.9
density: 1.0
```

**Ice Platform** (Static):
```yaml
body_type: "Static"
friction: 0.05
restitution: 0.0
```

**Trigger Zone** (Sensor):
```yaml
body_type: "Static"
is_sensor: true
```

### Collision Groups

Define which objects can collide with each other using collision groups and masks.

#### 1. Define Collision Groups

```rust
const PLAYER: Group = Group::GROUP_1;
const GROUND: Group = Group::GROUP_2;
const ENEMIES: Group = Group::GROUP_3;
const COLLECTIBLES: Group = Group::GROUP_4;
```

#### 2. Implement Parsing Function

```rust
fn parse_collision_layers(groups_str: &str, mask_str: &str) -> CollisionLayers {
    let mut memberships = Group::NONE;
    for group in groups_str.split(',').map(str::trim) {
        memberships |= match group {
            "player" => PLAYER,
            "ground" => GROUND,
            "enemies" => ENEMIES,
            _ => Group::NONE,
        };
    }

    let mut filters = Group::NONE;
    for group in mask_str.split(',').map(str::trim) {
        filters |= match group {
            "player" => PLAYER,
            "ground" => GROUND,
            "enemies" => ENEMIES,
            "all" => Group::ALL,
            _ => Group::NONE,
        };
    }

    CollisionLayers::new(memberships, filters)
}
```

#### 3. Register with Plugin

```rust
App::new()
    .add_plugins(BevyTiledAvianPlugin::new(
        PhysicsConfig {
            collision_layers_fn: parse_collision_layers,
            ..default()
        }
    ))
    .run();
```

#### 4. Configure in Tiled

```yaml
# Player object
physics_settings: avian::PhysicsSettings
  collision_groups: "player"
  collision_mask: "ground,enemies,collectibles"

# Enemy object
physics_settings: avian::PhysicsSettings
  collision_groups: "enemies"
  collision_mask: "ground,player"
```

**Collision Rule**: Objects collide if their groups/masks overlap bidirectionally.

## Tile Colliders

Generate physics colliders from tileset collision shapes automatically.

### Creating Collision Shapes in Tiled

1. Open your tileset in Tiled
2. Select a tile in the Tileset Editor
3. View → Tile Collision Editor (Ctrl+Shift+O)
4. Draw collision shapes using Rectangle, Polygon, or Polyline tools
5. Save the tileset

### Tile Collider Strategies

#### CompoundMerged (Recommended, Default)

Merges contiguous rectangular tiles into larger shapes for optimal performance.

- **Pros**: 5-100x collider reduction, excellent performance
- **Cons**: All tiles share one rigid body (can't move individually)
- **Use case**: Static terrain, walls, platforms

```rust
PhysicsConfig {
    tile_collider_strategy: TileColliderStrategy::CompoundMerged,
    ..default()
}
```

**Optimization Example**:
- Before: 100 tiles → 100 colliders
- After: 100 tiles → 10-20 merged rectangles

#### PerTileEntity

Spawns individual entities for each tile with a collision shape.

- **Pros**: Maximum flexibility (each tile can move independently)
- **Cons**: High entity count, poor performance for large maps
- **Use case**: Moving platforms, destructible terrain

```rust
PhysicsConfig {
    tile_collider_strategy: TileColliderStrategy::PerTileEntity,
    ..default()
}
```

#### CompoundChunked

Creates chunked compound colliders based on Tiled's chunk settings (default 16x16).

- **Pros**: Balance of performance and flexibility, enables spatial culling
- **Cons**: Multiple rigid bodies per layer
- **Use case**: Large/infinite maps, streaming levels

```rust
PhysicsConfig {
    tile_collider_strategy: TileColliderStrategy::CompoundChunked,
    ..default()
}
```

#### Disabled

No tile colliders generated (only object colliders).

```rust
PhysicsConfig {
    enable_tile_colliders: false,
    ..default()
}
```

## Examples

Run examples with:
```bash
cargo run --example <example_name>
```

- **`basic_physics`** - Simple object colliders with default settings
- **`debug_shapes`** - Visual debugging with AvianPhysicsDebug
- **`property_driven`** - Configure physics via Tiled properties
- **`collision_groups`** - Collision filtering with groups and masks
- **`tile_colliders`** - Automatic tile collider generation
- **`type_export`** - Export PhysicsSettings types to JSON for Tiled

## API Overview

### Components

- `PhysicsSettings` - Comprehensive physics configuration (TiledClass)
- `BodyType` - Enum for Static, Dynamic, Kinematic

### Resources

- `PhysicsConfig` - Global physics configuration and defaults

### Plugin

- `BevyTiledAvianPlugin` - Main plugin with optional custom config
  ```rust
  BevyTiledAvianPlugin::default()
  BevyTiledAvianPlugin::new(config)
  ```

## Shape Mapping

| Tiled Shape | Avian Collider |
|-------------|----------------|
| Rectangle   | `Collider::rectangle(w, h)` |
| Ellipse     | `Collider::circle(r)` (approximation) |
| Polygon     | `Collider::convex_hull()` or `Collider::trimesh()` |
| Polyline    | `Collider::polyline()` |
| Point       | `Collider::circle(1.0)` (small sensor) |
| Tile        | Extracted from tileset collision shapes |

## Troubleshooting

### Objects don't have colliders

**Issue**: Objects in Tiled don't generate physics colliders.

**Solution**: Objects ONLY get colliders if they have a `physics_settings` property. This is opt-in by design.

1. Add `physics_settings: avian::PhysicsSettings` property to objects in Tiled
2. Configure the nested physics parameters

### Tiles don't have colliders

**Issue**: Tiles don't generate physics colliders.

**Solution**:
1. Ensure `enable_tile_colliders: true` in PhysicsConfig (default)
2. Define collision shapes for tiles in the Tileset Editor (View → Tile Collision Editor)
3. Save the tileset and reload the map

### Collision groups not working

**Issue**: Objects pass through each other despite collision masks.

**Solution**:
1. Implement and register `collision_layers_fn` in PhysicsConfig
2. Verify collision_groups and collision_mask strings match your parsing function
3. Remember: Two objects collide if their groups/masks overlap **bidirectionally**

### Type autocomplete not working in Tiled

**Issue**: PhysicsSettings properties don't autocomplete in Tiled.

**Solution**:
1. Run `cargo run --example type_export` to generate `physics-types.json`
2. Import the file in Tiled (View → Custom Types Editor → Import)

## Compatibility

- Bevy: `0.17`
- Avian2D: `0.4`
- Tiled: `0.15`

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
