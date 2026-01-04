//! # `bevy_tiledmap_native` (Planned)
//!
//! This crate is a placeholder for future Bevy native tilemap rendering integration.
//!
//! ## Status: Not Yet Implemented
//!
//! This crate reserves the namespace for a future Layer 3 rendering plugin that will
//! use Bevy's upcoming native tilemap rendering system instead of `bevy_ecs_tilemap`.
//!
//! ## Planned Architecture
//!
//! When Bevy's native tilemap rendering is ready, this crate will:
//! - Observe the same events from `bevy_tiledmap_core` as `bevy_tiledmap_tilemap`
//! - Provide the same API surface (same features, similar configuration)
//! - Use Bevy's built-in tilemap components instead of `bevy_ecs_tilemap`
//! - Allow users to switch between rendering backends with minimal code changes
//!
//! ## Migration Path
//!
//! To switch from `bevy_tiledmap_tilemap` to `bevy_tiledmap_native` in the future:
//!
//! ```rust,ignore
//! // Instead of:
//! use bevy_tiledmap_tilemap::prelude::*;
//! app.add_plugins(TilemapPlugin::default());
//!
//! // Use:
//! use bevy_tiledmap_native::prelude::*;
//! app.add_plugins(TiledmapNativePlugin::default());
//! ```
//!
//! ## Timeline
//!
//! This crate will be implemented once Bevy's native tilemap rendering system is
//! stabilized and released. Track progress at:
//! - [Bevy Tilemap Tracking Issue](https://github.com/bevyengine/bevy/issues/...)
//!
//! ## Use `bevy_tiledmap_tilemap` Instead
//!
//! For production use today, use `bevy_tiledmap_tilemap` which provides full-featured,
//! high-performance tile rendering using `bevy_ecs_tilemap`.

use bevy::prelude::*;

/// Placeholder plugin for Bevy native tilemap rendering.
///
/// **This plugin is not yet implemented.** Use `TilemapPlugin` instead.
pub struct TiledmapNativePlugin;

impl Plugin for TiledmapNativePlugin {
    fn build(&self, _app: &mut App) {
        warn!(
            "bevy_tiledmap_native is not yet implemented! \
             Use bevy_tiledmap_tilemap for production rendering."
        );
    }
}

/// Prelude module (placeholder)
pub mod prelude {
    pub use crate::TiledmapNativePlugin;
}
