//! Plugin for `bevy_tiledmap_core`.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

use crate::debug::{DebugMapGeometry, draw_map_geometry_debug};
use crate::project::{TiledProjectAsset, TiledProjectProperties};
use crate::properties::{TiledClassRegistry, export_all_types_with_reflection};
use crate::systems::{check_world_spawn_complete, process_loaded_maps, process_loaded_worlds};

/// Configuration for layer Z-ordering.
///
/// Controls how layer Z values are calculated for proper rendering order.
/// Z value = `offset + (layer_index * multiplier)`
///
/// Groups don't contribute to Z - only actual content layers (tiles, objects, images)
/// are counted, giving flat Z-spacing across the entire layer hierarchy.
#[derive(Resource, Debug, Clone)]
pub struct LayerZConfig {
    /// Base Z offset for all layers
    pub offset: f32,
    /// Multiplier for layer index spacing
    pub multiplier: f32,
}

impl Default for LayerZConfig {
    fn default() -> Self {
        Self {
            offset: 0.0,
            multiplier: 1.0,
        }
    }
}

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
///         project_path: Some("assets/my.tiled-project".into()),
///         ..default()
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
    ///     ..Default::default()
    /// };
    /// ```
    pub export_types_path: Option<PathBuf>,

    /// Optional path to a `.tiled-project` file.
    ///
    /// If set, the plugin will load the project file and populate the
    /// `TiledProjectProperties` resource with custom property type definitions.
    /// This allows access to default values for classes and enum variants.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy_tiledmap_core::TiledmapCoreConfig;
    /// let config = TiledmapCoreConfig {
    ///     project_path: Some("assets/my.tiled-project".into()),
    ///     ..Default::default()
    /// };
    /// ```
    pub project_path: Option<PathBuf>,
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

/// Resource to track a pending project asset load
#[derive(Resource)]
struct PendingProjectLoad {
    handle: Handle<TiledProjectAsset>,
}

impl TiledmapCorePlugin {
    /// Create a new plugin with custom configuration.
    pub fn new(config: TiledmapCoreConfig) -> Self {
        Self { config }
    }
}

impl Plugin for TiledmapCorePlugin {
    fn build(&self, app: &mut App) {
        // Register the JSON asset plugin for .tiled-project files
        app.add_plugins(JsonAssetPlugin::<TiledProjectAsset>::new(&["tiled-project"]));

        // Initialize TiledProjectProperties resource (empty until project loads)
        app.init_resource::<TiledProjectProperties>();

        // Build the TiledClass registry from inventory
        let registry = TiledClassRegistry::build();

        // Insert registry as a resource
        app.insert_resource(registry);

        // Insert default layer Z config (can be overridden by user)
        app.init_resource::<LayerZConfig>();

        // Schedule type export at startup if configured
        // Must be done at startup to have access to AppTypeRegistry for reflection
        if let Some(path) = &self.config.export_types_path {
            app.insert_resource(DeferredTypeExport { path: path.clone() });
            app.add_systems(Startup, export_types_at_startup);
        }

        // Load project file if configured
        if let Some(project_path) = &self.config.project_path {
            let path = project_path.clone();
            app.add_systems(Startup, move |mut commands: Commands, asset_server: Res<AssetServer>| {
                let handle = asset_server.load::<TiledProjectAsset>(path.clone());
                commands.insert_resource(PendingProjectLoad { handle });
            });
            app.add_systems(PreUpdate, process_project_load.run_if(resource_exists::<PendingProjectLoad>));
        }

        // Add reactive spawning systems (runs in PreUpdate before user systems)
        // World processing runs before map processing so spawned maps get processed in the same frame
        // check_world_spawn_complete runs after maps are processed to fire WorldSpawned events
        app.add_systems(
            PreUpdate,
            (
                process_loaded_worlds,
                process_loaded_maps,
                check_world_spawn_complete,
            )
                .chain(),
        );

        // Enable debug visualization by default (remove this line to disable)
        app.init_resource::<DebugMapGeometry>();

        // Add debug visualization system (only runs when DebugMapGeometry resource is present)
        app.add_systems(
            PostUpdate,
            draw_map_geometry_debug.run_if(resource_exists::<DebugMapGeometry>),
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

/// System that processes a loaded project asset and populates TiledProjectProperties.
fn process_project_load(
    mut commands: Commands,
    pending: Res<PendingProjectLoad>,
    project_assets: Res<Assets<TiledProjectAsset>>,
    mut project_props: ResMut<TiledProjectProperties>,
) {
    // Check if the asset has finished loading
    let Some(asset) = project_assets.get(&pending.handle) else {
        return;
    };

    // Populate the TiledProjectProperties resource
    *project_props = TiledProjectProperties::from_asset(asset);

    info!(
        "Loaded Tiled project with {} classes and {} enums",
        project_props.classes().count(),
        project_props.enums().count()
    );

    // Remove the pending load marker
    commands.remove_resource::<PendingProjectLoad>();
}
