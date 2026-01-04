//! Plugin for `bevy_tiled_core`.

use bevy::prelude::*;

use crate::components::{
    LayerId, LayersInMap, ObjectId, ObjectsInMap, TiledLayer, TiledLayerMapOf, TiledMap,
    TiledObject, TiledObjectMapOf,
};
use crate::systems::process_loaded_maps;

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
///         .add_plugins(BevyTiledCorePlugin)
///         .run();
/// }
/// ```
pub struct BevyTiledCorePlugin;

impl Plugin for BevyTiledCorePlugin {
    fn build(&self, app: &mut App) {
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
