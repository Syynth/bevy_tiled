//! Image layer spawning.

use bevy::prelude::*;
use tiled::LayerType;

use crate::components::layer::ImageLayerData;
use crate::systems::SpawnContext;

/// Build `ImageLayerData` component from an image layer.
///
/// # Arguments
///
/// * `layer` - The image layer from the map asset
/// * `context` - Spawn context (for future image asset resolution)
///
/// # Returns
///
/// `ImageLayerData` component ready to attach to the layer entity
pub fn build_image_layer_data(
    layer: &tiled::Layer,
    _context: &SpawnContext,
) -> Option<ImageLayerData> {
    // Only process image layers
    let LayerType::Image(image_layer) = layer.layer_type() else {
        return None;
    };

    // Get image from the layer
    let image = image_layer.image.as_ref()?;

    // TODO: Resolve image path to Handle<Image>
    // For now, create a placeholder handle
    // In a real implementation, this would look up the image from the map asset's dependencies
    let image_handle = Handle::default();

    Some(ImageLayerData {
        image_handle,
        width: Some(image.width as f32),
        height: Some(image.height as f32),
    })
}
