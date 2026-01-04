//! Parallax scrolling for tile and image layers.
//!
//! Layers with `parallaxX` and `parallaxY` properties will move at different rates
//! relative to the camera, creating a depth effect.

use bevy::prelude::*;
use bevy_tiled_core::events::{ImageLayerSpawned, TileLayerSpawned};

// Re-export from bevy_tiled_core or use directly from tiled
use tiled::PropertyValue;

/// Marker component for the main camera that parallax layers follow.
///
/// Add this to your camera entity to enable parallax scrolling.
#[derive(Component, Debug, Default)]
pub struct ParallaxCamera;

/// Component that defines parallax behavior for a layer.
///
/// Lower values make the layer move slower (appear further away).
/// Values > 1.0 make the layer move faster (appear closer).
#[derive(Component, Debug, Clone)]
pub struct ParallaxLayer {
    /// Horizontal parallax factor (default: 1.0 = no parallax)
    pub parallax_x: f32,
    /// Vertical parallax factor (default: 1.0 = no parallax)
    pub parallax_y: f32,
    /// Cached previous camera position for delta calculation
    prev_camera_pos: Vec2,
}

impl Default for ParallaxLayer {
    fn default() -> Self {
        Self {
            parallax_x: 1.0,
            parallax_y: 1.0,
            prev_camera_pos: Vec2::ZERO,
        }
    }
}

impl ParallaxLayer {
    /// Create a new parallax layer with custom factors.
    pub fn new(parallax_x: f32, parallax_y: f32) -> Self {
        Self {
            parallax_x,
            parallax_y,
            prev_camera_pos: Vec2::ZERO,
        }
    }
}

/// Observer that checks tile layers for parallax properties and adds `ParallaxLayer` component.
pub fn add_parallax_to_tile_layer(trigger: On<TileLayerSpawned>, mut commands: Commands) {
    let event = trigger.event();

    // Check for parallax properties
    let parallax_x = event
        .properties
        .get("parallaxX")
        .and_then(|v| match v {
            PropertyValue::FloatValue(f) => Some(*f),
            PropertyValue::IntValue(i) => Some(*i as f32),
            _ => None,
        })
        .unwrap_or(1.0);

    let parallax_y = event
        .properties
        .get("parallaxY")
        .and_then(|v| match v {
            PropertyValue::FloatValue(f) => Some(*f),
            PropertyValue::IntValue(i) => Some(*i as f32),
            _ => None,
        })
        .unwrap_or(1.0);

    // Only add component if parallax is different from default (1.0)
    if (parallax_x - 1.0_f32).abs() > f32::EPSILON || (parallax_y - 1.0_f32).abs() > f32::EPSILON {
        commands
            .entity(event.entity)
            .insert(ParallaxLayer::new(parallax_x, parallax_y));
        info!(
            "Added parallax to tile layer ({}, {})",
            parallax_x, parallax_y
        );
    }
}

/// Observer that checks image layers for parallax properties and adds ParallaxLayer component.
pub fn add_parallax_to_image_layer(trigger: On<ImageLayerSpawned>, mut commands: Commands) {
    let event = trigger.event();

    // Check for parallax properties
    let parallax_x = event
        .properties
        .get("parallaxX")
        .and_then(|v| match v {
            PropertyValue::FloatValue(f) => Some(*f as f32),
            PropertyValue::IntValue(i) => Some(*i as f32),
            _ => None,
        })
        .unwrap_or(1.0);

    let parallax_y = event
        .properties
        .get("parallaxY")
        .and_then(|v| match v {
            PropertyValue::FloatValue(f) => Some(*f as f32),
            PropertyValue::IntValue(i) => Some(*i as f32),
            _ => None,
        })
        .unwrap_or(1.0);

    // Only add component if parallax is different from default (1.0)
    if (parallax_x - 1.0_f32).abs() > f32::EPSILON || (parallax_y - 1.0_f32).abs() > f32::EPSILON {
        commands
            .entity(event.entity)
            .insert(ParallaxLayer::new(parallax_x, parallax_y));
        info!(
            "Added parallax to image layer ({}, {})",
            parallax_x, parallax_y
        );
    }
}

/// System that updates parallax layer positions based on camera movement.
///
/// Moves layers with ParallaxLayer component based on the delta movement of
/// the ParallaxCamera, scaled by their parallax factors.
pub fn update_parallax_layers(
    camera_query: Query<&Transform, (With<ParallaxCamera>, Without<ParallaxLayer>)>,
    mut layer_query: Query<(&mut Transform, &mut ParallaxLayer)>,
) {
    // Get the camera position
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation.truncate();

    // Update all parallax layers
    for (mut layer_transform, mut parallax) in &mut layer_query {
        // Calculate delta movement since last frame
        let delta = camera_pos - parallax.prev_camera_pos;

        // Apply parallax factors to delta
        // Subtract 1.0 so that parallax_factor of 1.0 = no movement
        // parallax_factor of 0.5 = half speed (appears further away)
        // parallax_factor of 2.0 = double speed (appears closer)
        let parallax_delta_x = delta.x * (1.0 - parallax.parallax_x);
        let parallax_delta_y = delta.y * (1.0 - parallax.parallax_y);

        // Update layer position
        layer_transform.translation.x += parallax_delta_x;
        layer_transform.translation.y += parallax_delta_y;

        // Update cached camera position
        parallax.prev_camera_pos = camera_pos;
    }
}
