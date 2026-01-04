//! Main plugin for `bevy_tiledmap_tilemap`.

use bevy::prelude::*;

use crate::config::TilemapRenderConfig;
use crate::features::{z_ordering, ZOrderConfig};
use crate::images;
use crate::objects;
use crate::tiles;

#[cfg(feature = "animations")]
use crate::features::AnimationSpeed;

#[cfg(feature = "parallax")]
use crate::features::parallax;

/// Plugin for rendering Tiled maps with `bevy_ecs_tilemap`.
///
/// This Layer 3 plugin observes events from `bevy_tiledmap_core` and adds
/// rendering components to entities.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_tilemap::TilemapPlugin;
/// App::new()
///     .add_plugins(TilemapPlugin::default());
/// ```
#[derive(Default)]
pub struct TilemapPlugin {
    /// Configuration for rendering
    pub config: TilemapRenderConfig,
}


impl TilemapPlugin {
    /// Create plugin with custom configuration.
    pub fn new(config: TilemapRenderConfig) -> Self {
        Self { config }
    }
}

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        // Add bevy_ecs_tilemap plugin
        app.add_plugins(bevy_ecs_tilemap::TilemapPlugin);

        // Insert config resource
        app.insert_resource(self.config.clone());

        // Insert z-order config
        app.init_resource::<ZOrderConfig>();

        // Register tile layer rendering observer
        app.add_observer(tiles::render::on_tile_layer_spawned);

        // Register object rendering observer
        app.add_observer(objects::on_tile_object_spawned);

        // Register image layer rendering observer
        app.add_observer(images::on_image_layer_spawned);

        // Register z-ordering observers
        app.add_observer(z_ordering::set_tile_layer_z_order);
        app.add_observer(z_ordering::set_image_layer_z_order);
        app.add_observer(z_ordering::set_object_layer_z_order);
        app.add_observer(z_ordering::set_object_z_order);

        // Add animation systems if enabled
        #[cfg(feature = "animations")]
        if self.config.enable_animations {
            app.init_resource::<AnimationSpeed>();
            app.add_systems(Update, tiles::update_tile_animations);
        }

        // Add debug shape rendering if enabled
        #[cfg(feature = "debug_shapes")]
        if self.config.enable_debug_shapes {
            app.add_systems(Update, objects::render_object_shapes);
        }

        // Add parallax scrolling if enabled
        #[cfg(feature = "parallax")]
        if self.config.enable_parallax {
            app.add_observer(parallax::add_parallax_to_tile_layer);
            app.add_observer(parallax::add_parallax_to_image_layer);
            app.add_systems(Update, parallax::update_parallax_layers);
        }

        info!("TilemapPlugin initialized");
    }
}
