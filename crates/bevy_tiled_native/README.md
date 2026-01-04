# bevy_tiled_native

**Status: Not Yet Implemented - Placeholder Only**

This crate reserves the namespace for a future Layer 3 rendering plugin that will use Bevy's upcoming native tilemap rendering system.

## Why This Exists

The `bevy_tiled` architecture supports multiple rendering backends. This crate reserves the `bevy_tiled_native` namespace for when Bevy's built-in tilemap rendering is ready.

## Current Recommendation

**Use [`bevy_tiled_tilemap`](../bevy_tiled_tilemap) instead** for production rendering. It provides:
- Full-featured tile layer rendering with `bevy_ecs_tilemap`
- Tile animations
- Parallax scrolling
- Object and image layer rendering
- Excellent performance

## Future Plans

When Bevy's native tilemap rendering is stabilized, this crate will:
- Provide the same API as `bevy_tiled_tilemap`
- Use Bevy's built-in tilemap components
- Allow easy migration from `bevy_ecs_tilemap` backend
- Support the same Layer 3 features (animations, parallax, etc.)

## Migration (Future)

Switching between backends will be as simple as:

```rust
// Current (bevy_ecs_tilemap backend)
use bevy_tiled_tilemap::prelude::*;
app.add_plugins(BevyTiledTilemapPlugin::default());

// Future (Bevy native backend)
use bevy_tiled_native::prelude::*;
app.add_plugins(BevyTiledNativePlugin::default());
```

## Timeline

Implementation will begin once Bevy's native tilemap rendering system is released and stable.

## License

MIT OR Apache-2.0
