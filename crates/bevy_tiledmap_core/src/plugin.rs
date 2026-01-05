//! Plugin for `bevy_tiledmap_core`.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::properties::{TiledClassRegistry, export_all_types_with_reflection};
use crate::systems::{check_world_spawn_complete, process_loaded_maps, process_loaded_worlds};

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

/// Resource to store export path for deferred export
#[derive(Resource)]
struct DeferredTypeExport {
    path: PathBuf,
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

        // Insert registry as a resource
        app.insert_resource(registry);

        // Schedule type export at startup if configured
        // Must be done at startup to have access to AppTypeRegistry for reflection
        if let Some(path) = &self.config.export_types_path {
            app.insert_resource(DeferredTypeExport { path: path.clone() });
            app.add_systems(Startup, export_types_at_startup);
        }

        // Add reactive spawning systems (runs in PreUpdate before user systems)
        // World processing runs before map processing so spawned maps get processed in the same frame
        // check_world_spawn_complete runs after maps are processed to fire WorldSpawned events
        app.add_systems(
            PreUpdate,
            (process_loaded_worlds, process_loaded_maps, check_world_spawn_complete).chain(),
        );
    }
}

/// System that exports types at startup using reflection-based discovery
fn export_types_at_startup(world: &mut World) {
    let path = world
        .remove_resource::<DeferredTypeExport>()
        .expect("DeferredTypeExport resource should exist")
        .path;

    // export_all_types_with_reflection needs access to the world
    // We pass a reference to the world and handle it internally
    if let Err(e) = export_all_types_with_reflection(world, &path) {
        error!("Failed to export Tiled types to {}: {}", path.display(), e);
    } else {
        info!("Exported Tiled types to {}", path.display());
    }
}
