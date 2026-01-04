//! Tile animation component and update system.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::features::{AnimationSpeed, AnimationsPaused};

/// Component attached to animated tiles.
///
/// Contains the animation sequence and current playback state.
#[derive(Component, Debug, Clone)]
pub struct TileAnimation {
    /// Sequence of animation frames.
    pub frames: Vec<AnimationFrame>,
    /// Current frame index (`0..frames.len()`).
    pub current_frame: usize,
    /// Time elapsed in current frame (milliseconds).
    pub elapsed_ms: f32,
}

impl TileAnimation {
    /// Create a new tile animation from frame data.
    pub fn new(frames: Vec<AnimationFrame>) -> Self {
        Self {
            frames,
            current_frame: 0,
            elapsed_ms: 0.0,
        }
    }

    /// Get the current frame's tile ID.
    pub fn current_tile_id(&self) -> u32 {
        self.frames[self.current_frame].tile_id
    }

    /// Get the current frame's duration in milliseconds.
    pub fn current_duration_ms(&self) -> f32 {
        self.frames[self.current_frame].duration_ms as f32
    }

    /// Advance to the next frame, wrapping around.
    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        self.elapsed_ms = 0.0;
    }
}

/// A single frame in a tile animation.
#[derive(Debug, Clone, Copy)]
pub struct AnimationFrame {
    /// The tile ID to display for this frame.
    pub tile_id: u32,
    /// How long to display this frame (milliseconds).
    pub duration_ms: u32,
}

/// System that updates all animated tiles.
///
/// Advances animation frames based on elapsed time and updates `TileTextureIndex`.
pub fn update_tile_animations(
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    paused: Option<Res<AnimationsPaused>>,
    mut animated_tiles: Query<(&mut TileAnimation, &mut TileTextureIndex)>,
) {
    // Skip if animations are paused
    if paused.is_some() {
        return;
    }

    let delta_ms = time.delta_secs() * 1000.0 * speed.0;

    for (mut animation, mut texture_index) in &mut animated_tiles {
        animation.elapsed_ms += delta_ms;

        // Advance frames as needed
        while animation.elapsed_ms >= animation.current_duration_ms() {
            animation.next_frame();
            texture_index.0 = animation.current_tile_id();
        }
    }
}
