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
/// * `context` - Spawn context for image asset resolution
///
/// # Returns
///
/// `ImageLayerData` component ready to attach to the layer entity
pub fn build_image_layer_data(
    layer: &tiled::Layer,
    context: &SpawnContext,
) -> Option<ImageLayerData> {
    // Only process image layers
    let LayerType::Image(image_layer) = layer.layer_type() else {
        return None;
    };

    // Get image from the layer
    let image = image_layer.image.as_ref()?;

    // Look up the image handle from the map asset's images
    let image_handle = context.map_asset.images.get(&layer.id())?.clone();

    // Convert tiled Color to Bevy Color
    let tint_color = layer.tint_color.map(|c| {
        Color::srgba(
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            c.alpha as f32 / 255.0,
        )
    });

    // Calculate map pixel height for Layer 3 coordinate conversion
    let map_pixel_height = context.map_asset.map.height as f32 * context.map_asset.map.tile_height as f32;

    Some(ImageLayerData {
        image_handle,
        width: Some(image.width as f32),
        height: Some(image.height as f32),
        tint_color,
        map_pixel_height,
    })
}
