# Comprehensive Demo Plan

## Overview
A showcase example demonstrating all major features of `bevy_tiledmap` using a multi-map world with physics, rendering, and property system features.

## Demo Structure

### World Setup
- **Main World File** (`demo.world`): Contains references to 3 different maps
  - Map 1: Gameplay area with physics and interactions
  - Map 2: Parallax background showcase
  - Map 3: Animated tiles showcase

---

## Features to Showcase

### 1. Asset Loading (Layer 1)
- [x] World file loading (`.world`)
- [x] Multiple map files (`.tmx`)
- [x] External tilesets (`.tsx`)
- [x] Template files (`.tx`)
- [x] Image collections vs sprite sheets

### 2. Core Features (Layer 2)

#### Property System
- [x] **TiledClass custom properties** with various types:
  - `GameplaySettings` - Float values, booleans
  - `LootSettings` - Enums, integers
  - `SpawnSettings` - String values
  - `PhysicsSettings` - Complex nested properties (from avian)

- [x] **Property inheritance chain**:
  - Map properties → Layer properties → Object properties
  - Tileset → Tile → Object (for tile objects)
  - Template → Object instance

- [x] **ClassValue properties** (the new type-safe system)

#### Entity Hierarchy
- [x] Map entities
- [x] Layer entities (tile, object, image, group)
- [x] Object entities (all 7 types)
- [x] Relationships for bidirectional traversal

#### Layer Types
- [x] Tile layers (standard rectangular grids)
- [x] Object layers (with various object types)
- [x] Image layers (parallax backgrounds)
- [x] Group layers (organizational)

#### Object Types (All 7)
- [x] Rectangle
- [x] Ellipse
- [x] Polygon
- [x] Polyline
- [x] Point
- [x] Tile (from tileset)
- [x] Text (for UI/signs)

### 3. Tilemap Rendering (Layer 3 - `tilemap` feature)

#### Tile Rendering
- [x] Multi-tileset layers (single layer using multiple tilesets)
- [x] Different tile sizes (16x16, 32x32)
- [x] Isometric rendering (optional - if we want to get fancy)

#### Animations
- [x] Animated tiles (water, torches, flags)
- [x] Different animation speeds
- [x] Frame-based animations from tilesets

#### Visual Effects
- [x] Parallax scrolling (background layers)
- [x] Layer opacity/transparency
- [x] Z-ordering (proper depth sorting)
- [x] Image layers as backgrounds

#### Debug Visualization
- [x] Debug shapes for collision geometry
- [x] Object shape rendering

### 4. Physics Integration (Layer 3 - `avian` feature)

#### Object Colliders (All Shapes)
- [x] **Rectangle**: Platforms, walls, boxes
- [x] **Ellipse**: Circular obstacles, boulders
- [x] **Polygon**: Custom terrain shapes
- [x] **Polyline**: Ramps, slopes (non-filled shapes)
- [x] **Point**: Trigger zones, spawn points (small sensors)
- [x] **Tile Objects**: Objects that reference tileset tiles with collision

#### Tile Colliders
- [x] Tileset collision shapes (rectangular)
- [x] Custom tile collision polygons
- [x] Rectangle merging optimization (showing performance improvement)
- [x] Different tile collider strategies:
  - `CompoundMerged` (default - optimized)
  - `PerTileEntity` (flexible)

#### Physics Configuration
- [x] **Static bodies**: Terrain, platforms, walls
- [x] **Dynamic bodies**: Player, enemies, movable objects
- [x] **Kinematic bodies**: Moving platforms

#### Material Properties
- [x] **Friction**: Ice (0.0), Normal (0.5), Rubber (1.0)
- [x] **Restitution**: Bouncy (0.8), Normal (0.0)
- [x] **Density**: Heavy (10.0), Normal (1.0), Light (0.1)

#### Advanced Physics
- [x] **Sensors**: Trigger zones that don't collide physically
- [x] **Collision groups/layers**:
  - Player vs Environment
  - Player vs Enemies
  - Projectiles vs Walls
- [x] **Linear/Angular damping**: For realistic movement
- [x] **Gravity scale**: Per-object gravity multipliers
- [x] **Rotation locking**: For characters that shouldn't rotate

### 5. Templates

Template files demonstrating reusability:
- [x] `player_spawn.tx` - Spawn point template with properties
- [x] `enemy_patrol.tx` - Enemy with patrol AI properties
- [x] `loot_chest.tx` - Interactive chest with loot settings
- [x] `moving_platform.tx` - Kinematic platform with path data

### 6. Custom Properties (TiledClass Examples)

```rust
// Gameplay configuration
#[derive(TiledClass)]
struct GameplaySettings {
    player_speed: f32,
    jump_height: f32,
    enable_double_jump: bool,
}

// Loot system
#[derive(TiledClass)]
struct LootSettings {
    loot_tier: LootTier,  // Enum: Common, Rare, Epic, Legendary
    quantity: i32,
    respawn_time: f32,
}

// Enemy AI
#[derive(TiledClass)]
struct EnemySettings {
    enemy_type: EnemyType,  // Enum: Melee, Ranged, Flying
    patrol_radius: f32,
    aggro_range: f32,
    health: i32,
}

// Spawn system
#[derive(TiledClass)]
struct SpawnSettings {
    spawn_group: String,  // "player", "enemy", "item"
    is_random_spawn: bool,
    max_spawns: i32,
}
```

---

## Map Breakdown

### Map 1: "Gameplay Arena" (`maps/gameplay.tmx`)

**Purpose**: Showcase physics, collisions, and gameplay features

**Layers**:
1. **Background** (Tile Layer)
   - Simple background tiles
   - Parallax offset property

2. **Terrain** (Tile Layer)
   - Ground tiles with collision shapes
   - Shows rectangle merging (100+ tiles → 5 merged colliders)
   - Mix of solid and platform tiles

3. **Platforms** (Tile Layer)
   - One-way platforms
   - Moving platform tiles

4. **Decoration** (Tile Layer)
   - Non-collision decorative tiles
   - Animated torches, flags

5. **Collision Objects** (Object Layer)
   - Rectangle walls
   - Ellipse boulders
   - Polygon custom terrain
   - Polyline ramps/slopes

6. **Interactive** (Object Layer - uses templates)
   - Player spawn (point with template)
   - Enemy spawns (with EnemySettings)
   - Loot chests (with LootSettings)
   - Trigger zones (sensors)

7. **Foreground** (Tile Layer)
   - Foreground decoration (higher z-order)

**Tilesets**:
- `terrain_16x16.tsx` - Ground, walls with collision
- `decoration_16x16.tsx` - Non-collision decorative tiles
- `animated_16x16.tsx` - Water, torches with animations

### Map 2: "Parallax Showcase" (`maps/parallax.tmx`)

**Purpose**: Demonstrate parallax scrolling and visual effects

**Layers**:
1. **Sky** (Image Layer)
   - Static background image
   - Parallax factor: 0.0

2. **Far Mountains** (Tile Layer)
   - Parallax factor: 0.2
   - Large mountain tiles

3. **Mid Mountains** (Tile Layer)
   - Parallax factor: 0.5

4. **Near Trees** (Tile Layer)
   - Parallax factor: 0.8

5. **Ground** (Tile Layer)
   - Parallax factor: 1.0 (normal)
   - With collision

**Tilesets**:
- `background_32x32.tsx` - Large background tiles
- `terrain_16x16.tsx` - Ground tiles

### Map 3: "Animation Gallery" (`maps/animations.tmx`)

**Purpose**: Showcase animated tiles and tile variety

**Layers**:
1. **Base** (Tile Layer)
   - Static ground

2. **Water** (Tile Layer)
   - Animated water tiles (4 frames, 500ms each)

3. **Flames** (Tile Layer)
   - Torch animations (6 frames, 150ms each)

4. **Flags** (Tile Layer)
   - Flag animations (3 frames, 300ms each)

5. **Misc Animated** (Tile Layer)
   - Various other animated elements

**Tilesets**:
- `animated_16x16.tsx` - All animated tiles

---

## Tilesets

### 1. `terrain_16x16.tsx`
- **Size**: 16x16 pixels
- **Tiles**: ~50 tiles
- **Features**:
  - Collision shapes on solid tiles
  - Mix of full-tile and partial collision
  - Different material properties (ice, normal, sticky)

### 2. `decoration_16x16.tsx`
- **Size**: 16x16 pixels
- **Tiles**: ~30 tiles
- **Features**:
  - No collision shapes
  - Pure visual decoration

### 3. `animated_16x16.tsx`
- **Size**: 16x16 pixels
- **Tiles**: ~20 animated tiles
- **Features**:
  - Water (4 frames)
  - Torch (6 frames)
  - Flag (3 frames)
  - Gears, crystals, etc.

### 4. `background_32x32.tsx`
- **Size**: 32x32 pixels
- **Tiles**: ~20 tiles
- **Features**:
  - Large background elements
  - Mountains, clouds, trees

---

## Templates

### 1. `player_spawn.tx`
```xml
<template>
  <object name="PlayerSpawn" type="Point" width="16" height="16">
    <properties>
      <property name="spawn_settings" type="class" propertytype="SpawnSettings">
        <properties>
          <property name="spawn_group" value="player"/>
          <property name="is_random_spawn" value="false"/>
        </properties>
      </property>
    </properties>
  </object>
</template>
```

### 2. `enemy_patrol.tx`
```xml
<template>
  <object name="Enemy" type="Ellipse" width="16" height="16">
    <properties>
      <property name="physics_settings" type="class" propertytype="avian::PhysicsSettings">
        <properties>
          <property name="body_type" value="Dynamic"/>
          <property name="friction" value="0.5"/>
          <property name="density" value="1.0"/>
        </properties>
      </property>
      <property name="enemy_settings" type="class" propertytype="EnemySettings">
        <properties>
          <property name="enemy_type" value="Melee"/>
          <property name="patrol_radius" value="100"/>
          <property name="health" value="10"/>
        </properties>
      </property>
    </properties>
  </object>
</template>
```

### 3. `loot_chest.tx`
```xml
<template>
  <object name="Chest" type="Rectangle" width="16" height="16">
    <properties>
      <property name="physics_settings" type="class" propertytype="avian::PhysicsSettings">
        <properties>
          <property name="body_type" value="Static"/>
          <property name="is_sensor" value="true"/>
        </properties>
      </property>
      <property name="loot_settings" type="class" propertytype="LootSettings">
        <properties>
          <property name="loot_tier" value="Common"/>
          <property name="quantity" value="3"/>
        </properties>
      </property>
    </properties>
  </object>
</template>
```

---

## Example Structure

### File: `examples/comprehensive_demo.rs`

```rust
use bevy::prelude::*;
use bevy_tiledmap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(
            BevyTiledmapPlugin::default()
                .with_core(TiledmapCoreConfig {
                    // Export type definitions for Tiled
                    export_types_path: Some("assets/tiled_types.json".into()),
                })
                .with_tilemap(TilemapRenderConfig {
                    enable_animations: true,
                    enable_parallax: true,
                    enable_debug_shapes: true,
                })
                .with_avian(PhysicsConfig {
                    enable_tile_colliders: true,
                    collision_layers_fn: parse_collision_layers,
                    ..default()
                })
        )
        // Register custom TiledClass types
        .register_type::<GameplaySettings>()
        .register_type::<LootSettings>()
        .register_type::<EnemySettings>()
        .register_type::<SpawnSettings>()
        .add_systems(Startup, (setup_camera, load_world))
        .add_systems(Update, (
            handle_player_spawns,
            handle_enemy_spawns,
            handle_loot_chests,
            show_feature_info,
        ))
        .run();
}
```

---

## Success Metrics

After implementing this demo, we should be able to:

1. **Load the world** and see all 3 maps accessible
2. **See tile rendering** with multiple tilesets
3. **See animations** playing smoothly
4. **See parallax** working on background layers
5. **See physics colliders** generated from both objects and tiles
6. **See collision merging** reducing collider count by 10-100x
7. **Read custom properties** from TiledClass system
8. **Use templates** for consistent object configuration
9. **Export type definitions** to `tiled_types.json` for Tiled autocomplete
10. **Test collision groups** working correctly

---

## Assets Needed

### To Create in Tiled:
- [ ] `demo.world` - World file
- [ ] `maps/gameplay.tmx` - Main gameplay map
- [ ] `maps/parallax.tmx` - Parallax showcase map
- [ ] `maps/animations.tmx` - Animation gallery map
- [ ] `tilesets/terrain_16x16.tsx` - Terrain tileset with collision
- [ ] `tilesets/decoration_16x16.tsx` - Decorative tileset
- [ ] `tilesets/animated_16x16.tsx` - Animated tiles
- [ ] `tilesets/background_32x32.tsx` - Background elements
- [ ] `templates/player_spawn.tx` - Player spawn template
- [ ] `templates/enemy_patrol.tx` - Enemy template
- [ ] `templates/loot_chest.tx` - Loot template

### Image Assets (can use placeholder/existing):
- [ ] terrain_16x16.png
- [ ] decoration_16x16.png
- [ ] animated_16x16.png
- [ ] background_32x32.png
- [ ] sky_background.png (for image layer)

---

## Implementation Order

1. **Create TiledClass structs** in example code
2. **Run example** to generate `tiled_types.json`
3. **Create tilesets** in Tiled with collision shapes
4. **Create templates** in Tiled using exported types
5. **Create maps** in Tiled using tilesets and templates
6. **Create world file** linking all maps
7. **Finish example code** with spawn handlers and systems
8. **Test all features** and verify everything works

---

This comprehensive demo will serve as both a showcase and a real-world example of how to use `bevy_tiledmap` effectively!
