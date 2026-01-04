//! Plugin for `Avian2D` physics integration.

use bevy::prelude::*;

use crate::config::PhysicsConfig;
use crate::objects;
use crate::tiles;

/// Plugin that integrates `Avian2D` physics with `bevy_tiled`.
///
/// This plugin:
/// - Registers the [`PhysicsConfig`] resource for global configuration
/// - Adds observers for object collider generation
/// - Optionally adds observers for tile collider generation (if enabled)
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiled_avian::{BevyTiledAvianPlugin, PhysicsConfig};
/// use avian2d::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(PhysicsPlugins::default())
///     .add_plugins(BevyTiledAvianPlugin::default())
///     .run();
/// ```
///
/// # Custom Configuration
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiled_avian::{BevyTiledAvianPlugin, PhysicsConfig};
/// use avian2d::prelude::*;
///
/// App::new()
///     .add_plugins(BevyTiledAvianPlugin::new(
///         PhysicsConfig {
///             default_friction: 0.3,
///             ..default()
///         }
///     ))
///     .run();
/// ```
#[derive(Default)]
pub struct BevyTiledAvianPlugin {
    /// Physics configuration
    pub config: PhysicsConfig,
}

impl BevyTiledAvianPlugin {
    /// Create a new plugin with custom configuration.
    pub fn new(config: PhysicsConfig) -> Self {
        Self { config }
    }
}


impl Plugin for BevyTiledAvianPlugin {
    fn build(&self, app: &mut App) {
        // Insert resources
        app.insert_resource(self.config.clone());

        // Register types for reflection
        app.register_type::<crate::properties::PhysicsSettings>();
        app.register_type::<crate::properties::BodyType>();

        // Add observers for object colliders
        app.add_observer(objects::on_object_spawned);

        // Add observers for tile colliders if enabled
        if self.config.enable_tile_colliders {
            app.add_observer(tiles::on_tile_layer_spawned);
        }

        info!("BevyTiledAvianPlugin initialized");
    }
}
