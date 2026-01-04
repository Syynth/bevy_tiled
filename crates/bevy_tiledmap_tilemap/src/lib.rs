//! # `bevy_tiledmap_tilemap`
//!
//! High-performance tile layer rendering for `bevy_tiled` using `bevy_ecs_tilemap`.
//!
//! This crate is a Layer 3 plugin that observes spawning events from `bevy_tiledmap_core`
//! and adds rendering components using `bevy_ecs_tilemap` for optimal batched rendering.
//!
//! ## Features
//!
//! - **Tile layers**: Batched rendering with `bevy_ecs_tilemap`
//! - **Multi-tileset support**: Handles layers using multiple tilesets
//! - **Tile animations**: Automatic frame cycling based on tileset animation data
//! - **Object rendering**: Sprites for tile objects, debug shapes for collision geometry
//! - **Image layers**: Simple sprite rendering
//! - **Parallax scrolling**: Layer parallax based on Tiled properties
//! - **Z-ordering**: Automatic depth sorting
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_tiledmap_assets::BevyTiledAssetsPlugin;
//! use bevy_tiledmap_core::prelude::*;
//! use bevy_tiledmap_tilemap::BevyTiledTilemapPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BevyTiledAssetsPlugin)
//!         .add_plugins(BevyTiledCorePlugin::default())
//!         .add_plugins(BevyTiledTilemapPlugin::default())
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

pub mod config;
pub mod features;
pub mod images;
pub mod objects;
pub mod plugin;
pub mod tiles;

pub use config::TilemapRenderConfig;
pub use plugin::TilemapPlugin;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::config::TilemapRenderConfig;
    pub use crate::features::{AnimationSpeed, AnimationsPaused, ParallaxCamera, ZOrderConfig};
    pub use crate::plugin::TilemapPlugin;
}
