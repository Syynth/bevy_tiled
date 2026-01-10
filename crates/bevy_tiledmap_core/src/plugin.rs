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

/// Target for type export.
///
/// Specifies where to export the registered `TiledClass` types.
#[derive(Debug, Clone)]
pub enum TypeExportTarget {
    /// Export to a standalone JSON file.
    ///
    /// The JSON file will contain an array of property type definitions
    /// that can be imported into Tiled via View → Custom Types → Import.
    JsonFile(PathBuf),

    /// Export directly to the configured `.tiled-project` file.
    ///
    /// This will update the `propertyTypes` array in the project file while
    /// preserving all other fields (folders, commands, etc.). Requires
    /// `project_path` to be set in the config.
    ///
    /// Any manually-added types in the project file that don't match
    /// a Rust type will be preserved.
    TiledProject,
}

/// Configuration for `TiledmapCorePlugin`.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap_core::{TiledmapCorePlugin, TiledmapCoreConfig, TypeExportTarget};
///
/// App::new()
///     .add_plugins(TiledmapCorePlugin::new(TiledmapCoreConfig {
///         export_target: Some(TypeExportTarget::JsonFile("tiled_types.json".into())),
///         project_path: Some("my.tiled-project".into()),
///         ..default()
///     }));
/// ```
#[derive(Debug, Clone)]
pub struct TiledmapCoreConfig {
    /// Where to export type definitions for Tiled editor.
    ///
    /// If set, the plugin will export all registered `TiledClass` types at startup.
    /// Use `TypeExportTarget::JsonFile` to export to a standalone JSON file, or
    /// `TypeExportTarget::TiledProject` to export directly to the `.tiled-project` file.
    ///
    /// Paths are relative to the asset root directory.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy_tiledmap_core::{TiledmapCoreConfig, TypeExportTarget};
    /// // Export to a standalone JSON file
    /// let config = TiledmapCoreConfig {
    ///     export_target: Some(TypeExportTarget::JsonFile("tiled_types.json".into())),
    ///     ..Default::default()
    /// };
    ///
    /// // Or export directly to the project file
    /// let config = TiledmapCoreConfig {
    ///     export_target: Some(TypeExportTarget::TiledProject),
    ///     project_path: Some("my.tiled-project".into()),
    ///     ..Default::default()
    /// };
    /// ```
    pub export_target: Option<TypeExportTarget>,

    /// Optional path to a `.tiled-project` file (relative to asset root).
    ///
    /// If set, the plugin will load the project file and populate the
    /// `TiledProjectProperties` resource with custom property type definitions.
    /// This allows access to default values for classes and enum variants.
    ///
    /// Also required when using `TypeExportTarget::TiledProject`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy_tiledmap_core::TiledmapCoreConfig;
    /// let config = TiledmapCoreConfig {
    ///     project_path: Some("my.tiled-project".into()),
    ///     ..Default::default()
    /// };
    /// ```
    pub project_path: Option<PathBuf>,

    /// The root directory for assets (filesystem path).
    ///
    /// This should match your `AssetPlugin::file_path` configuration.
    /// Defaults to "assets" (Bevy's default).
    pub asset_root: PathBuf,
}

impl Default for TiledmapCoreConfig {
    fn default() -> Self {
        Self {
            export_target: None,
            project_path: None,
            asset_root: PathBuf::from("assets"),
        }
    }
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
/// use bevy_tiledmap_core::{TiledmapCorePlugin, TiledmapCoreConfig, TypeExportTarget};
///
/// App::new()
///     .add_plugins(TiledmapCorePlugin::new(TiledmapCoreConfig {
///         export_target: Some(TypeExportTarget::JsonFile("tiled_types.json".into())),
///         ..default()
///     }));
/// ```
#[derive(Default)]
pub struct TiledmapCorePlugin {
    config: TiledmapCoreConfig,
}

/// Resource to store export configuration for deferred export
#[derive(Resource)]
struct DeferredTypeExport {
    target: TypeExportTarget,
    project_path: Option<PathBuf>,
    asset_root: PathBuf,
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
        app.add_plugins(JsonAssetPlugin::<TiledProjectAsset>::new(&[
            "tiled-project",
        ]));

        // Initialize TiledProjectProperties resource (empty until project loads)
        app.init_resource::<TiledProjectProperties>();

        // Build the TiledClass registry from inventory
        let registry = TiledClassRegistry::build();

        // Insert registry as a resource
        app.insert_resource(registry);

        // Insert default layer Z config (can be overridden by user)
        app.init_resource::<LayerZConfig>();

        // Initialize world Z counters for shared layer Z-ordering across maps
        app.init_resource::<crate::systems::spawn::WorldZCounters>();

        // Schedule type export at startup if configured
        // Must be done at startup to have access to AppTypeRegistry for reflection
        if let Some(target) = &self.config.export_target {
            app.insert_resource(DeferredTypeExport {
                target: target.clone(),
                project_path: self.config.project_path.clone(),
                asset_root: self.config.asset_root.clone(),
            });
            app.add_systems(Startup, export_types_at_startup);
        }

        // Load project file if configured
        if let Some(project_path) = &self.config.project_path {
            let path = project_path.clone();
            app.add_systems(
                Startup,
                move |mut commands: Commands, asset_server: Res<AssetServer>| {
                    let handle = asset_server.load::<TiledProjectAsset>(path.clone());
                    commands.insert_resource(PendingProjectLoad { handle });
                },
            );
            app.add_systems(
                PreUpdate,
                process_project_load.run_if(resource_exists::<PendingProjectLoad>),
            );
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

        // Add debug visualization system (only runs when DebugMapGeometry resource is present)
        app.add_systems(
            PostUpdate,
            draw_map_geometry_debug.run_if(resource_exists::<DebugMapGeometry>),
        );
    }
}

/// System that exports types at startup using reflection-based discovery
fn export_types_at_startup(world: &mut World) {
    use crate::properties::export_to_tiled_project;

    let deferred = world
        .remove_resource::<DeferredTypeExport>()
        .expect("DeferredTypeExport resource should exist");

    let result = match &deferred.target {
        TypeExportTarget::JsonFile(path) => {
            let full_path = deferred.asset_root.join(path);
            export_all_types_with_reflection(world, &full_path)
                .map(|_| format!("Exported Tiled types to {}", full_path.display()))
        }
        TypeExportTarget::TiledProject => {
            let project_path = deferred
                .project_path
                .as_ref()
                .expect("project_path is required for TypeExportTarget::TiledProject");
            let full_path = deferred.asset_root.join(project_path);
            export_to_tiled_project(world, &full_path)
                .map(|_| format!("Exported Tiled types to {}", full_path.display()))
        }
    };

    match result {
        Ok(msg) => info!("{}", msg),
        Err(e) => error!("Failed to export Tiled types: {}", e),
    }
}

/// System that processes a loaded project asset and populates `TiledProjectProperties`.
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
