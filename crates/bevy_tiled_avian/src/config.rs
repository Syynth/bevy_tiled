//! Global physics configuration and defaults.

use avian2d::prelude::*;
use bevy::prelude::*;

/// Global physics configuration resource.
///
/// This resource controls default physics parameters for all colliders generated from Tiled maps.
/// These defaults are used when objects don't have a `physics_settings` property.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_tiled_avian::{BevyTiledAvianPlugin, PhysicsConfig};
/// use avian2d::prelude::*;
///
/// // Define collision groups
/// const PLAYER: Group = Group::GROUP_1;
/// const GROUND: Group = Group::GROUP_2;
///
/// fn parse_collision_layers(groups: &str, mask: &str) -> CollisionLayers {
///     let mut memberships = Group::NONE;
///     for group in groups.split(',').map(str::trim) {
///         memberships |= match group {
///             "player" => PLAYER,
///             "ground" => GROUND,
///             _ => Group::NONE,
///         };
///     }
///
///     let mut filters = Group::NONE;
///     for group in mask.split(',').map(str::trim) {
///         filters |= match group {
///             "player" => PLAYER,
///             "ground" => GROUND,
///             "all" => Group::ALL,
///             _ => Group::NONE,
///         };
///     }
///
///     CollisionLayers::new(memberships, filters)
/// }
///
/// App::new()
///     .add_plugins(BevyTiledAvianPlugin::new(
///         PhysicsConfig {
///             default_friction: 0.3,
///             collision_layers_fn: parse_collision_layers,
///             ..default()
///         }
///     ))
///     .run();
/// ```
#[derive(Resource, Clone)]
pub struct PhysicsConfig {
    /// Default friction coefficient (0.0 = no friction, 1.0 = high friction).
    ///
    /// Default: `0.5`
    pub default_friction: f32,

    /// Default restitution coefficient (0.0 = no bounce, 1.0 = perfect bounce).
    ///
    /// Default: `0.0`
    pub default_restitution: f32,

    /// Default density for dynamic bodies (kg/m²).
    ///
    /// Default: `1.0`
    pub default_density: f32,

    /// Default rigid body type.
    ///
    /// Default: [`RigidBody::Static`]
    pub default_body_type: RigidBody,

    /// Default sensor flag (sensors don't generate collision responses).
    ///
    /// Default: `false`
    pub default_is_sensor: bool,

    /// Default collision layers for objects without explicit collision groups.
    ///
    /// Default: [`CollisionLayers::default()`]
    pub default_collision_layers: CollisionLayers,

    /// User-provided function to convert string collision groups/mask to Avian's `CollisionLayers`.
    ///
    /// This callback is used to parse the `collision_groups` and `collision_mask` strings from
    /// Tiled properties into Avian's [`CollisionLayers`] type.
    ///
    /// # Arguments
    ///
    /// * `groups` - Comma-separated collision group memberships (e.g., `"player,friendly"`)
    /// * `mask` - Comma-separated collision mask/filters (e.g., `"ground,enemies"`)
    ///
    /// # Returns
    ///
    /// An Avian [`CollisionLayers`] struct configured with the appropriate groups and filters.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use avian2d::prelude::*;
    ///
    /// const PLAYER: Group = Group::GROUP_1;
    /// const GROUND: Group = Group::GROUP_2;
    ///
    /// fn my_collision_fn(groups: &str, mask: &str) -> CollisionLayers {
    ///     let mut memberships = Group::NONE;
    ///     for group in groups.split(',').map(str::trim) {
    ///         memberships |= match group {
    ///             "player" => PLAYER,
    ///             "ground" => GROUND,
    ///             _ => Group::NONE,
    ///         };
    ///     }
    ///
    ///     let mut filters = Group::ALL; // Default to colliding with everything
    ///     if !mask.is_empty() {
    ///         filters = Group::NONE;
    ///         for group in mask.split(',').map(str::trim) {
    ///             filters |= match group {
    ///                 "player" => PLAYER,
    ///                 "ground" => GROUND,
    ///                 "all" => Group::ALL,
    ///                 _ => Group::NONE,
    ///             };
    ///         }
    ///     }
    ///
    ///     CollisionLayers::new(memberships, filters)
    /// }
    /// ```
    pub collision_layers_fn: fn(&str, &str) -> CollisionLayers,

    /// Enable automatic tile collider generation from tileset collision shapes.
    ///
    /// When enabled, the plugin will generate colliders for tiles that have collision
    /// shapes defined in their tileset.
    ///
    /// Default: `true`
    pub enable_tile_colliders: bool,

    /// Strategy for generating tile colliders.
    ///
    /// Default: [`TileColliderStrategy::CompoundMerged`]
    pub tile_collider_strategy: TileColliderStrategy,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            default_friction: 0.5,
            default_restitution: 0.0,
            default_density: 1.0,
            default_body_type: RigidBody::Static,
            default_is_sensor: false,
            default_collision_layers: CollisionLayers::default(),
            collision_layers_fn: default_collision_layers_fn,
            enable_tile_colliders: true,
            tile_collider_strategy: TileColliderStrategy::CompoundMerged,
        }
    }
}

/// Default collision layers function (no-op).
///
/// This function is used when no custom collision layers function is provided.
/// It simply returns the default collision layers.
fn default_collision_layers_fn(_groups: &str, _mask: &str) -> CollisionLayers {
    CollisionLayers::default()
}

impl PhysicsConfig {
    /// Builder method: Set default friction.
    pub fn with_default_friction(mut self, friction: f32) -> Self {
        self.default_friction = friction;
        self
    }

    /// Builder method: Set default restitution.
    pub fn with_default_restitution(mut self, restitution: f32) -> Self {
        self.default_restitution = restitution;
        self
    }

    /// Builder method: Set default density.
    pub fn with_default_density(mut self, density: f32) -> Self {
        self.default_density = density;
        self
    }

    /// Builder method: Set default body type.
    pub fn with_default_body_type(mut self, body_type: RigidBody) -> Self {
        self.default_body_type = body_type;
        self
    }

    /// Builder method: Set default sensor flag.
    pub fn with_default_is_sensor(mut self, is_sensor: bool) -> Self {
        self.default_is_sensor = is_sensor;
        self
    }

    /// Builder method: Set default collision layers.
    pub fn with_default_collision_layers(mut self, collision_layers: CollisionLayers) -> Self {
        self.default_collision_layers = collision_layers;
        self
    }

    /// Builder method: Set collision layers conversion function.
    pub fn with_collision_layers_fn(
        mut self,
        collision_layers_fn: fn(&str, &str) -> CollisionLayers,
    ) -> Self {
        self.collision_layers_fn = collision_layers_fn;
        self
    }

    /// Builder method: Enable or disable tile colliders.
    pub fn with_tile_colliders(mut self, enable: bool) -> Self {
        self.enable_tile_colliders = enable;
        self
    }

    /// Builder method: Set tile collider strategy.
    pub fn with_tile_collider_strategy(mut self, strategy: TileColliderStrategy) -> Self {
        self.tile_collider_strategy = strategy;
        self
    }
}

/// Strategy for generating tile colliders from tileset collision shapes.
///
/// Different strategies offer trade-offs between performance and flexibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileColliderStrategy {
    /// Don't generate tile colliders.
    ///
    /// Use this when you only want object colliders, or when you're handling
    /// tile colliders manually.
    Disabled,

    /// Spawn individual entities for each tile with a collision shape.
    ///
    /// **Pros:**
    /// - Maximum flexibility (each tile can move independently)
    /// - Tiles can have different physics properties
    ///
    /// **Cons:**
    /// - High entity count
    /// - Poor performance for large collision layers
    ///
    /// **Use case:** Moving platforms, destructible terrain
    PerTileEntity,

    /// Create optimized compound collider with rectangle merging (Recommended).
    ///
    /// Follows `bevy_ecs_tiled` approach: merges contiguous rectangular tiles into larger shapes.
    ///
    /// **Pros:**
    /// - Excellent performance
    /// - Drastically reduced collider count (5-100x reduction)
    ///
    /// **Cons:**
    /// - All tiles in layer share one rigid body (can't move individually)
    ///
    /// **Use case:** Static terrain, walls, platforms
    CompoundMerged,

    /// Create chunked compound colliders based on Tiled's chunk settings.
    ///
    /// Combines chunking with rectangle merging optimization.
    ///
    /// **Pros:**
    /// - Balance of performance and flexibility
    /// - Enables spatial culling
    /// - Aligns with Tiled's internal organization
    ///
    /// **Cons:**
    /// - More complex
    /// - Multiple rigid bodies per layer
    ///
    /// **Use case:** Large/infinite maps, streaming levels
    CompoundChunked,
}
