//! Plugin for `bevy_tiled_core`.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::components::{
    LayerId, LayersInMap, ObjectId, ObjectsInMap, TiledLayer, TiledLayerMapOf, TiledMap,
    TiledObject, TiledObjectMapOf,
};
use crate::properties::{TiledClassRegistry, export_types_to_json};
use crate::systems::process_loaded_maps;

/// Configuration for `BevyTiledCorePlugin`.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiled_core::{BevyTiledCorePlugin, BevyTiledCoreConfig};
///
/// App::new()
///     .add_plugins(BevyTiledCorePlugin::new(BevyTiledCoreConfig {
///         export_types_path: Some("assets/tiled_types.json".into()),
///     }));
/// ```
#[derive(Debug, Clone, Default)]
pub struct BevyTiledCoreConfig {
    /// Optional path to export type definitions for Tiled editor.
    ///
    /// If set, the plugin will export all registered `TiledClass` types to a JSON file
    /// at startup. This file can be used by Tiled to provide autocomplete for custom
    /// properties.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy_tiled_core::BevyTiledCoreConfig;
    /// let config = BevyTiledCoreConfig {
    ///     export_types_path: Some("assets/tiled_types.json".into()),
    /// };
    /// ```
    pub export_types_path: Option<PathBuf>,
}

/// Plugin for the `bevy_tiled_core` entity spawning system.
///
/// Add this plugin after `BevyTiledAssetsPlugin` to enable automatic map spawning.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiled_assets::BevyTiledAssetsPlugin;
/// use bevy_tiled_core::BevyTiledCorePlugin;
///
/// fn app() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(BevyTiledAssetsPlugin)
///         .add_plugins(BevyTiledCorePlugin::default())
///         .run();
/// }
/// ```
///
/// # With Configuration
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiled_core::{BevyTiledCorePlugin, BevyTiledCoreConfig};
///
/// App::new()
///     .add_plugins(BevyTiledCorePlugin::new(BevyTiledCoreConfig {
///         export_types_path: Some("assets/tiled_types.json".into()),
///     }));
/// ```
#[derive(Default)]
pub struct BevyTiledCorePlugin {
    config: BevyTiledCoreConfig,
}

impl BevyTiledCorePlugin {
    /// Create a new plugin with custom configuration.
    pub fn new(config: BevyTiledCoreConfig) -> Self {
        Self { config }
    }
}

impl Plugin for BevyTiledCorePlugin {
    fn build(&self, app: &mut App) {
        // Build the TiledClass registry from inventory
        let registry = TiledClassRegistry::build();

        // Export types to JSON if configured
        if let Some(path) = &self.config.export_types_path {
            if let Err(e) = export_types_to_json(&registry, path) {
                error!("Failed to export Tiled types to {}: {}", path.display(), e);
            } else {
                info!("Exported Tiled types to {}", path.display());
            }
        }

        // Insert registry as a resource
        app.insert_resource(registry);

        // Register types for reflection
        app.register_type::<TiledMap>()
            .register_type::<TiledLayer>()
            .register_type::<LayerId>()
            .register_type::<LayersInMap>()
            .register_type::<TiledLayerMapOf>()
            .register_type::<ObjectsInMap>()
            .register_type::<TiledObjectMapOf>()
            .register_type::<ObjectId>()
            .register_type::<TiledObject>();

        // Add reactive spawning system (runs in PreUpdate before user systems)
        app.add_systems(PreUpdate, process_loaded_maps);
    }
}
