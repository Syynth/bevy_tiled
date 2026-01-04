//! # bevy_tiledmap
//!
//! Tiled map loader and integration for Bevy.
//!
//! This is a unified meta-crate that combines all `bevy_tiledmap_*` sub-crates with convenient
//! feature flags for easy integration.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_tiledmap::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BevyTiledmapPlugin::default())
//!         .add_systems(Startup, spawn_map)
//!         .run();
//! }
//!
//! fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.spawn(TiledMap {
//!         handle: asset_server.load("map.tmx"),
//!     });
//! }
//! ```
//!
//! ## Features
//!
//! - **default**: Includes `tilemap` feature for rendering
//! - **tilemap**: Tile layer rendering using `bevy_ecs_tilemap` (recommended)
//! - **avian**: Physics collider generation using `avian2d`
//! - **native**: Bevy native tilemap rendering (placeholder for future)
//!
//! ## Architecture
//!
//! This crate is organized into 3 layers:
//!
//! - **Layer 1** ([`assets`]): Pure asset loading for Tiled files (.tmx, .tsx, .tx, .world)
//! - **Layer 2** ([`core`]): ECS entity spawning with property merging and events
//! - **Layer 3** (optional): Integration plugins for rendering and physics
//!   - [`tilemap`]: High-performance tilemap rendering with `bevy_ecs_tilemap`
//!   - [`avian`]: Physics integration with Avian2D
//!   - [`native`]: Future Bevy native tilemap support (placeholder)
//!
//! ## Using Individual Crates
//!
//! You can also use the individual sub-crates directly if you prefer more control:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_tiledmap_assets::TiledmapAssetsPlugin;
//! use bevy_tiledmap_core::prelude::*;
//! use bevy_tiledmap_tilemap::TilemapPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(TiledmapAssetsPlugin)
//!     .add_plugins(TiledmapCorePlugin::default())
//!     .add_plugins(TilemapPlugin::default())
//!     .run();
//! ```

pub mod plugin;

// Re-export sub-crates for advanced usage
pub use bevy_tiledmap_assets as assets;
pub use bevy_tiledmap_core as core;

#[cfg(feature = "tilemap")]
pub use bevy_tiledmap_tilemap as tilemap;

#[cfg(feature = "avian")]
pub use bevy_tiledmap_avian as avian;

#[cfg(feature = "native")]
pub use bevy_tiledmap_native as native;

/// Unified prelude for bevy_tiledmap
///
/// This module re-exports the most commonly used types from all sub-crates
/// for convenient access.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap::prelude::*;
///
/// fn my_system(
///     maps: Query<&TiledMap>,
///     objects: Query<&TiledObject>,
/// ) {
///     // Work with Tiled entities...
/// }
/// ```
pub mod prelude {
    // Core functionality (always available)
    pub use crate::assets::prelude::*;
    pub use crate::core::prelude::*;

    // Layer 3 plugins (feature-gated)
    #[cfg(feature = "tilemap")]
    pub use crate::tilemap::prelude::*;

    #[cfg(feature = "avian")]
    pub use crate::avian::prelude::*;

    #[cfg(feature = "native")]
    pub use crate::native::prelude::*;

    // Unified plugin
    pub use crate::plugin::BevyTiledmapPlugin;
}
