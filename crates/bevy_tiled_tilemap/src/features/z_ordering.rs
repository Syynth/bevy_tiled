//! Z-ordering for layers and objects.
//!
//! Automatically sets Transform.z values based on layer IDs and hierarchy.

use bevy::prelude::*;
use bevy_tiled_core::events::{ImageLayerSpawned, ObjectLayerSpawned, ObjectSpawned, TileLayerSpawned};

/// Configuration for z-ordering.
///
/// Controls how layers and objects are positioned in depth.
#[derive(Resource, Debug, Clone)]
pub struct ZOrderConfig {
    /// Z-coordinate separation between layers (default: 10.0)
    ///
    /// Each layer gets z = layer_id * layer_separation
    pub layer_separation: f32,

    /// Z-offset for objects above their parent layer (default: 1.0)
    ///
    /// Objects get z = parent_layer_z + object_z_offset
    pub object_z_offset: f32,
}

impl Default for ZOrderConfig {
    fn default() -> Self {
        Self {
            layer_separation: 10.0,
            object_z_offset: 1.0,
        }
    }
}

/// Observer that sets z-order for tile layers.
///
/// Sets Transform.z = layer_id * layer_separation
pub fn set_tile_layer_z_order(
    trigger: On<TileLayerSpawned>,
    config: Res<ZOrderConfig>,
    mut transform_query: Query<&mut Transform>,
) {
    let event = trigger.event();

    if let Ok(mut transform) = transform_query.get_mut(event.entity) {
        transform.translation.z = event.layer_id as f32 * config.layer_separation;
    }
}

/// Observer that sets z-order for image layers.
///
/// Sets Transform.z = layer_id * layer_separation
pub fn set_image_layer_z_order(
    trigger: On<ImageLayerSpawned>,
    config: Res<ZOrderConfig>,
    mut transform_query: Query<&mut Transform>,
) {
    let event = trigger.event();

    if let Ok(mut transform) = transform_query.get_mut(event.entity) {
        transform.translation.z = event.layer_id as f32 * config.layer_separation;
    }
}

/// Observer that sets z-order for object layers.
///
/// Sets Transform.z = layer_id * layer_separation
pub fn set_object_layer_z_order(
    trigger: On<ObjectLayerSpawned>,
    config: Res<ZOrderConfig>,
    mut transform_query: Query<&mut Transform>,
) {
    let event = trigger.event();

    if let Ok(mut transform) = transform_query.get_mut(event.entity) {
        transform.translation.z = event.layer_id as f32 * config.layer_separation;
    }
}

/// Observer that sets z-order for objects relative to their parent layer.
///
/// Objects inherit their parent layer's z and add object_z_offset
pub fn set_object_z_order(
    trigger: On<ObjectSpawned>,
    config: Res<ZOrderConfig>,
    parent_query: Query<&ChildOf>,
    layer_transform_query: Query<&Transform, Without<ChildOf>>,
    mut object_transform_query: Query<&mut Transform, With<ChildOf>>,
) {
    let event = trigger.event();

    // Get the object's parent (the object layer)
    let Ok(parent) = parent_query.get(event.entity) else {
        return;
    };

    // Get the parent layer's z-coordinate
    // ChildOf is a tuple struct wrapping the parent Entity
    let parent_z = layer_transform_query
        .get(parent.0)
        .map(|t| t.translation.z)
        .unwrap_or(0.0);

    // Set object's z to be slightly above the layer
    if let Ok(mut transform) = object_transform_query.get_mut(event.entity) {
        transform.translation.z = parent_z + config.object_z_offset;
    }
}
