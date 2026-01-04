//! # bevy_tiled_native (Planned)
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
//! - Observe the same events from `bevy_tiled_core` as `bevy_tiled_tilemap`
//! - Provide the same API surface (same features, similar configuration)
//! - Use Bevy's built-in tilemap components instead of `bevy_ecs_tilemap`
//! - Allow users to switch between rendering backends with minimal code changes
//!
//! ## Migration Path
//!
//! To switch from `bevy_tiled_tilemap` to `bevy_tiled_native` in the future:
//!
//! ```rust,ignore
//! // Instead of:
//! use bevy_tiled_tilemap::prelude::*;
//! app.add_plugins(BevyTiledTilemapPlugin::default());
//!
//! // Use:
//! use bevy_tiled_native::prelude::*;
//! app.add_plugins(BevyTiledNativePlugin::default());
//! ```
//!
//! ## Timeline
//!
//! This crate will be implemented once Bevy's native tilemap rendering system is
//! stabilized and released. Track progress at:
//! - [Bevy Tilemap Tracking Issue](https://github.com/bevyengine/bevy/issues/...)
//!
//! ## Use bevy_tiled_tilemap Instead
//!
//! For production use today, use `bevy_tiled_tilemap` which provides full-featured,
//! high-performance tile rendering using `bevy_ecs_tilemap`.

use bevy::prelude::*;

/// Placeholder plugin for Bevy native tilemap rendering.
///
/// **This plugin is not yet implemented.** Use `BevyTiledTilemapPlugin` instead.
pub struct BevyTiledNativePlugin;

impl Plugin for BevyTiledNativePlugin {
    fn build(&self, _app: &mut App) {
        warn!(
            "bevy_tiled_native is not yet implemented! \
             Use bevy_tiled_tilemap for production rendering."
        );
    }
}

/// Prelude module (placeholder)
pub mod prelude {
    pub use crate::BevyTiledNativePlugin;
}
