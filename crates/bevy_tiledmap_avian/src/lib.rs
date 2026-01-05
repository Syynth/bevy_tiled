//! `Avian2D` physics integration for `bevy_tiled`.
//!
//! This crate provides automatic collider generation from Tiled maps using the `Avian2D` physics engine.
//!
//! # Features
//!
//! - **Object Colliders**: Generate colliders from Tiled objects (Rectangle, Ellipse, Polygon, Polyline, Point, Tile)
//! - **Tile Colliders**: Generate optimized colliders from tileset collision shapes with rectangle merging
//! - **Property-Based Configuration**: Configure physics parameters via `PhysicsSettings` `TiledClass`
//! - **Collision Layers**: User-provided callback for converting string collision groups to Avian's `CollisionLayers`
//! - **Multiple Strategies**: Choose between `PerTileEntity`, `CompoundMerged`, or `CompoundChunked` for tile colliders
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_tiledmap_avian::{TiledmapAvianPlugin, PhysicsConfig};
//! use avian2d::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(PhysicsPlugins::default())
//!     .add_plugins(TiledmapAvianPlugin::default())
//!     .run();
//! ```
//!
//! # Custom Configuration
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use bevy_tiledmap_avian::{TiledmapAvianPlugin, PhysicsConfig};
//! use avian2d::prelude::*;
//!
//! // Define collision groups
//! const PLAYER: Group = Group::GROUP_1;
//! const GROUND: Group = Group::GROUP_2;
//!
//! fn parse_collision_layers(groups: &str, mask: &str) -> CollisionLayers {
//!     // Parse comma-separated strings into Avian's CollisionLayers
//!     // ... implementation ...
//!     CollisionLayers::default()
//! }
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(PhysicsPlugins::default())
//!     .add_plugins(TiledmapAvianPlugin::new(
//!         PhysicsConfig {
//!             default_friction: 0.3,
//!             collision_layers_fn: parse_collision_layers,
//!             ..default()
//!         }
//!     ))
//!     .run();
//! ```

pub mod config;
pub mod objects;
pub mod plugin;
pub mod properties;
pub mod shapes;
pub mod tiles;

pub mod prelude {
    //! Common imports for `bevy_tiledmap_avian`.

    pub use crate::config::*;
    pub use crate::plugin::TiledmapAvianPlugin;
    pub use crate::properties::*;
}

// Re-export at crate root for convenience
pub use config::PhysicsConfig;
pub use plugin::TiledmapAvianPlugin;
pub use properties::{BodyType, PhysicsSettings};
