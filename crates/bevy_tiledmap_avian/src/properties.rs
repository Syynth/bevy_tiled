//! Physics property types for Tiled integration.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiledmap_macros::TiledClass;

use crate::config::PhysicsConfig;

/// Comprehensive physics settings for Tiled objects.
///
/// This struct can be used as a custom class in Tiled to configure physics parameters
/// for individual objects. Add this to your Tiled object as a `physics_settings` property
/// with type `avian::PhysicsSettings`.
///
/// # Example in Tiled
///
/// Create a custom class property on an object:
/// ```text
/// Property name: physics_settings
/// Property type: avian::PhysicsSettings
///
/// Values:
///   body_type: "Dynamic"
///   friction: 0.8
///   restitution: 0.3
///   collision_groups: "player"
///   collision_mask: "ground,enemies"
/// ```
///
/// # Field Reference
///
/// - `body_type`: Static, Dynamic, or Kinematic
/// - `friction`: 0.0 (no friction) to 1.0+ (high friction)
/// - `restitution`: 0.0 (no bounce) to 1.0 (perfect bounce)
/// - `density`: Mass per unit area (kg/m²) for dynamic bodies
/// - `collision_groups`: Comma-separated group memberships (e.g., "player,friendly")
/// - `collision_mask`: Comma-separated collision filters (e.g., "ground,enemies")
/// - `is_sensor`: If true, detects collisions but doesn't generate responses
/// - `linear_damping`: Reduces linear velocity over time
/// - `angular_damping`: Reduces angular velocity over time
/// - `gravity_scale`: Multiplier for gravity (0.0 = no gravity, 2.0 = double gravity)
/// - `lock_rotation`: Prevents rotation if true
#[derive(Component, Reflect, TiledClass, Debug, Clone)]
#[reflect(Component)]
#[tiled(name = "avian::PhysicsSettings")]
pub struct PhysicsSettings {
    /// Rigid body type (Static, Dynamic, or Kinematic).
    ///
    /// Default: "Static"
    #[tiled(default = BodyType::Static)]
    pub body_type: BodyType,

    /// Friction coefficient (0.0 = no friction, 1.0 = high friction).
    ///
    /// Default: 0.5
    #[tiled(default = 0.5)]
    pub friction: f32,

    /// Restitution coefficient (0.0 = no bounce, 1.0 = perfect bounce).
    ///
    /// Default: 0.0
    #[tiled(default = 0.0)]
    pub restitution: f32,

    /// Density for dynamic bodies (kg/m²).
    ///
    /// Default: 1.0
    #[tiled(default = 1.0)]
    pub density: f32,

    /// Collision group memberships (comma-separated string).
    ///
    /// Example: "player,friendly"
    ///
    /// The `PhysicsConfig`'s `collision_layers_fn` callback converts this string
    /// to Avian's `CollisionLayers` type.
    ///
    /// Default: "" (empty = use default collision layers from `PhysicsConfig`)
    #[tiled(default = String::new())]
    pub collision_groups: String,

    /// Collision mask/filters (comma-separated string).
    ///
    /// Example: "ground,enemies,all"
    ///
    /// Which groups this object collides with. The `PhysicsConfig`'s
    /// `collision_layers_fn` callback converts this to Avian's `CollisionLayers`.
    ///
    /// Default: "" (empty = use default collision layers from `PhysicsConfig`)
    #[tiled(default = String::new())]
    pub collision_mask: String,

    /// Sensor flag - detects collisions but doesn't generate responses.
    ///
    /// Default: false
    #[tiled(default = false)]
    pub is_sensor: bool,

    /// Linear damping - reduces linear velocity over time.
    ///
    /// Default: None (no damping)
    pub linear_damping: Option<f32>,

    /// Angular damping - reduces angular velocity over time.
    ///
    /// Default: None (no damping)
    pub angular_damping: Option<f32>,

    /// Gravity scale multiplier (0.0 = no gravity, 2.0 = double gravity).
    ///
    /// Default: None (use default gravity)
    pub gravity_scale: Option<f32>,

    /// Lock rotation - prevents this object from rotating.
    ///
    /// Default: false
    #[tiled(default = false)]
    pub lock_rotation: bool,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            body_type: BodyType::Static,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            collision_groups: String::new(),
            collision_mask: String::new(),
            is_sensor: false,
            linear_damping: None,
            angular_damping: None,
            gravity_scale: None,
            lock_rotation: false,
        }
    }
}

impl PhysicsSettings {
    /// Convert collision groups/mask strings to Avian's `CollisionLayers`.
    ///
    /// Uses the user-provided callback from `PhysicsConfig` to parse the
    /// comma-separated strings into Avian's `CollisionLayers` type.
    ///
    /// If both strings are empty, returns the default collision layers
    /// from `PhysicsConfig`.
    pub fn collision_layers(&self, config: &PhysicsConfig) -> CollisionLayers {
        if self.collision_groups.is_empty() && self.collision_mask.is_empty() {
            // Use default
            config.default_collision_layers
        } else {
            // Call user-provided conversion function
            (config.collision_layers_fn)(&self.collision_groups, &self.collision_mask)
        }
    }

    /// Convert to Avian's `RigidBody` type.
    pub fn to_rigid_body(&self) -> RigidBody {
        match self.body_type {
            BodyType::Static => RigidBody::Static,
            BodyType::Dynamic => RigidBody::Dynamic,
            BodyType::Kinematic => RigidBody::Kinematic,
        }
    }
}

/// Rigid body type for physics objects.
///
/// This enum is used in `PhysicsSettings` to specify the body type.
#[derive(Reflect, TiledClass, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[tiled(name = "avian::BodyType")]
pub enum BodyType {
    /// Static body - doesn't move, infinite mass.
    ///
    /// Use for: walls, platforms, terrain
    #[default]
    Static,

    /// Dynamic body - affected by forces and gravity.
    ///
    /// Use for: player, enemies, movable objects
    Dynamic,

    /// Kinematic body - moves but not affected by forces.
    ///
    /// Use for: moving platforms, elevators, scripted animations
    Kinematic,
}
