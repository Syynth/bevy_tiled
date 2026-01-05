//! Plugin for `bevy_tiledmap_core`.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::properties::{TiledClassRegistry, export_types_to_json};
use crate::systems::{process_loaded_maps, process_loaded_worlds};

/// Configuration for `TiledmapCorePlugin`.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap_core::{TiledmapCorePlugin, TiledmapCoreConfig};
///
/// App::new()
///     .add_plugins(TiledmapCorePlugin::new(TiledmapCoreConfig {
///         export_types_path: Some("assets/tiled_types.json".into()),
///     }));
/// ```
#[derive(Debug, Clone, Default)]
pub struct TiledmapCoreConfig {
    /// Optional path to export type definitions for Tiled editor.
    ///
    /// If set, the plugin will export all registered `TiledClass` types to a JSON file
    /// at startup. This file can be used by Tiled to provide autocomplete for custom
    /// properties.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy_tiledmap_core::TiledmapCoreConfig;
    /// let config = TiledmapCoreConfig {
    ///     export_types_path: Some("assets/tiled_types.json".into()),
    /// };
    /// ```
    pub export_types_path: Option<PathBuf>,
}

/// Plugin for the `bevy_tiledmap_core` entity spawning system.
///
/// Add this plugin after `TiledmapAssetsPlugin` to enable automatic map spawning.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap_assets::TiledmapAssetsPlugin;
/// use bevy_tiledmap_core::TiledmapCorePlugin;
///
/// fn app() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(TiledmapAssetsPlugin)
///         .add_plugins(TiledmapCorePlugin::default())
///         .run();
/// }
/// ```
///
/// # With Configuration
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap_core::{TiledmapCorePlugin, TiledmapCoreConfig};
///
/// App::new()
///     .add_plugins(TiledmapCorePlugin::new(TiledmapCoreConfig {
///         export_types_path: Some("assets/tiled_types.json".into()),
///     }));
/// ```
#[derive(Default)]
pub struct TiledmapCorePlugin {
    config: TiledmapCoreConfig,
}

impl TiledmapCorePlugin {
    /// Create a new plugin with custom configuration.
    pub fn new(config: TiledmapCoreConfig) -> Self {
        Self { config }
    }
}

impl Plugin for TiledmapCorePlugin {
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

        // Add reactive spawning systems (runs in PreUpdate before user systems)
        // World processing runs before map processing so spawned maps get processed in the same frame
        app.add_systems(
            PreUpdate,
            (process_loaded_worlds, process_loaded_maps).chain(),
        );
    }
}
