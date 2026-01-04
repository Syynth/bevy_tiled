//! Unified plugin for bevy_tiledmap.

use bevy::prelude::*;

use bevy_tiledmap_assets::TiledmapAssetsPlugin;
use bevy_tiledmap_core::{TiledmapCoreConfig, TiledmapCorePlugin};

#[cfg(feature = "tilemap")]
use bevy_tiledmap_tilemap::{TilemapPlugin, TilemapRenderConfig};

#[cfg(feature = "avian")]
use bevy_tiledmap_avian::{PhysicsConfig, TiledmapAvianPlugin};

#[cfg(feature = "native")]
use bevy_tiledmap_native::TiledmapNativePlugin;

/// Unified plugin that adds all enabled bevy_tiledmap functionality.
///
/// This plugin automatically includes:
/// - Asset loading ([`TiledmapAssetsPlugin`])
/// - Core ECS spawning ([`TiledmapCorePlugin`])
/// - Enabled Layer 3 integrations based on feature flags
///
/// # Features
///
/// - `tilemap` (default): Adds [`TilemapPlugin`] for rendering with bevy_ecs_tilemap
/// - `avian`: Adds [`TiledmapAvianPlugin`] for Avian2D physics integration
/// - `native`: Adds [`TiledmapNativePlugin`] (placeholder for future Bevy native tilemap)
///
/// # Example
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(BevyTiledmapPlugin::default())
///     .run();
/// ```
///
/// # With Custom Configuration
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_tiledmap::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(
///         BevyTiledmapPlugin::default()
///             .with_core(TiledmapCoreConfig {
///                 export_types_path: Some("assets/tiled_types.json".into()),
///             })
///     )
///     .run();
/// ```
#[derive(Default)]
pub struct BevyTiledmapPlugin {
    /// Core configuration
    pub core: TiledmapCoreConfig,

    /// Tilemap rendering configuration (if feature enabled)
    #[cfg(feature = "tilemap")]
    pub tilemap: TilemapRenderConfig,

    /// Avian physics configuration (if feature enabled)
    #[cfg(feature = "avian")]
    pub avian: PhysicsConfig,
}

impl BevyTiledmapPlugin {
    /// Create with custom core configuration
    pub fn with_core(mut self, config: TiledmapCoreConfig) -> Self {
        self.core = config;
        self
    }

    /// Create with custom tilemap rendering configuration
    #[cfg(feature = "tilemap")]
    pub fn with_tilemap(mut self, config: TilemapRenderConfig) -> Self {
        self.tilemap = config;
        self
    }

    /// Create with custom Avian physics configuration
    #[cfg(feature = "avian")]
    pub fn with_avian(mut self, config: PhysicsConfig) -> Self {
        self.avian = config;
        self
    }
}

impl Plugin for BevyTiledmapPlugin {
    fn build(&self, app: &mut App) {
        // Layer 1: Assets (always required)
        app.add_plugins(TiledmapAssetsPlugin);

        // Layer 2: Core (always required)
        app.add_plugins(TiledmapCorePlugin::new(self.core.clone()));

        // Layer 3: Rendering (feature-gated)
        #[cfg(feature = "tilemap")]
        app.add_plugins(TilemapPlugin::new(self.tilemap.clone()));

        #[cfg(feature = "native")]
        app.add_plugins(TiledmapNativePlugin);

        // Layer 3: Physics (feature-gated)
        #[cfg(feature = "avian")]
        app.add_plugins(TiledmapAvianPlugin::new(self.avian.clone()));

        info!("BevyTiledmapPlugin initialized");
    }
}
