//! # `bevy_tiled_core`
//!
//! Entity spawning backbone for `bevy_tiled`. Converts loaded Tiled assets into structured
//! ECS hierarchies with property merging, relationships, and extension hooks.
//!
//! **This crate does NOT handle rendering or physics** - those are Layer 3 concerns that
//! plug in via events and component queries.
//!
//! ## Architecture
//!
//! Layer 2 (this crate) sits between:
//! - **Layer 1** (`bevy_tiled_assets`): Pure asset loading
//! - **Layer 3** (`bevy_tiled_render`, `bevy_tiled_avian`, etc.): Rendering/physics plugins
//!
//! ## What Layer 2 Provides
//!
//! 1. **Entity hierarchy**: Maps, layers, and objects (NOT individual tiles)
//! 2. **Pre-processed data**: `TileLayerData` with tile grid, pre-computed object vertices
//! 3. **Relationships**: Bevy relationship system for bidirectional traversal
//! 4. **Events**: Extension hooks for Layer 3 plugins
//!
//! ## What Layer 2 Does NOT Provide
//!
//! - Individual tile entities (only `TileLayerData` component)
//! - Rendering components (Sprite, `TilemapBundle`, etc.)
//! - Physics components (Collider, `RigidBody`, etc.)
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_tiled_assets::BevyTiledAssetsPlugin;
//! use bevy_tiled_core::{BevyTiledCorePlugin, TiledMap};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BevyTiledAssetsPlugin)
//!         .add_plugins(BevyTiledCorePlugin)
//!         .add_systems(Startup, spawn_map)
//!         .run();
//! }
//!
//! fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.spawn(TiledMap {
//!         handle: asset_server.load("maps/level1.tmx"),
//!     });
//! }
//! ```

pub mod components;
pub mod events;
pub mod plugin;
pub mod properties;
pub mod spawn;
pub mod systems;

pub mod prelude {
    //! Common imports for `bevy_tiled_core` users.

    pub use crate::components::{
        LayerId, ObjectId, TiledLayer, TiledLayerMapOf, TiledMap, TiledObject, TiledObjectMapOf,
    };
    pub use crate::events::{
        ImageLayerCreated, MapCreated, ObjectCreated, ObjectLayerCreated, TileLayerCreated,
        TiledEvent,
    };
    pub use crate::plugin::BevyTiledCorePlugin;
    pub use crate::properties::MergedProperties;
}

// Re-export plugin at crate root for convenience
pub use plugin::BevyTiledCorePlugin;
