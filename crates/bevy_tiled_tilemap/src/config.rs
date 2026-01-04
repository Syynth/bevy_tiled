//! Configuration for tilemap rendering.

use bevy::prelude::*;

/// Configuration for tilemap rendering plugin.
#[derive(Resource, Clone, Debug)]
pub struct TilemapRenderConfig {
    /// Enable tile animations (default: true with "animations" feature)
    pub enable_animations: bool,

    /// Enable parallax scrolling (default: true with "parallax" feature)
    pub enable_parallax: bool,

    /// Enable debug shape rendering with gizmos (default: false)
    pub enable_debug_shapes: bool,
}

impl Default for TilemapRenderConfig {
    fn default() -> Self {
        Self {
            enable_animations: cfg!(feature = "animations"),
            enable_parallax: cfg!(feature = "parallax"),
            enable_debug_shapes: cfg!(feature = "debug_shapes"),
        }
    }
}
