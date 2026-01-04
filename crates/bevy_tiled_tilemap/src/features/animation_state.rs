//! Global resources for controlling tile animations.

use bevy::prelude::*;

/// Global speed multiplier for all tile animations.
///
/// Default is 1.0 (normal speed). Set to 2.0 for double speed, 0.5 for half speed.
#[derive(Resource, Debug, Clone)]
pub struct AnimationSpeed(pub f32);

impl Default for AnimationSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Resource that pauses all tile animations when present.
///
/// Insert this resource to pause animations, remove it to resume.
#[derive(Resource, Debug, Default, Clone)]
pub struct AnimationsPaused;
