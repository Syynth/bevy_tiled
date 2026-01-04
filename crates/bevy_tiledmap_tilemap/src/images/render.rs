//! Observer for image layer spawning events.

use bevy::prelude::*;
use bevy_tiledmap_core::components::layer::ImageLayerData;
use bevy_tiledmap_core::events::ImageLayerSpawned;

/// Observer that renders image layers as sprites.
///
/// When an image layer is spawned by Layer 2, this observer:
/// 1. Reads the `ImageLayerData` component
/// 2. Creates a Sprite with the image
/// 3. Sets custom size if width/height are specified
pub fn on_image_layer_spawned(
    trigger: On<ImageLayerSpawned>,
    layer_query: Query<&ImageLayerData>,
    mut transform_query: Query<&mut Transform>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok(image_data) = layer_query.get(event.entity) else {
        warn!(
            "ImageLayerSpawned event for entity {:?} but no ImageLayerData component found",
            event.entity
        );
        return;
    };

    info!("Rendering image layer entity {:?}", event.entity);

    // Calculate custom size or scale
    if let (Some(width), Some(height)) = (image_data.width, image_data.height) {
        // If custom dimensions are specified, scale the sprite
        if let Some(image) = images.get(&image_data.image_handle) {
            let image_size = image.size_f32();
            let scale = Vec2::new(width / image_size.x, height / image_size.y);

            // Update existing transform's scale
            if let Ok(mut transform) = transform_query.get_mut(event.entity) {
                transform.scale = scale.extend(1.0);
            }
        }
    }

    // Insert sprite component
    commands.entity(event.entity).insert(Sprite {
        image: image_data.image_handle.clone(),
        ..default()
    });

    info!("Created sprite for image layer");
}
