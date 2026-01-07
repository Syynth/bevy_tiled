//! Observer for image layer spawning events.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_tiledmap_core::components::layer::ImageLayerData;
use bevy_tiledmap_core::events::ImageLayerSpawned;

/// Observer that renders image layers as sprites.
///
/// When an image layer is spawned by Layer 2, this observer:
/// 1. Reads the `ImageLayerData` component
/// 2. Creates a Sprite with the image
/// 3. Adjusts transform to use Bevy coordinates (positive Y)
/// 4. Sets anchor to BottomLeft (images extend up and right in Bevy's Y-up space)
pub fn on_image_layer_spawned(
    trigger: On<ImageLayerSpawned>,
    layer_query: Query<(&ImageLayerData, &Transform, Option<&Name>)>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok((image_data, transform, name)) = layer_query.get(event.entity) else {
        warn!(
            "ImageLayerSpawned event for entity {:?} but no ImageLayerData component found",
            event.entity
        );
        return;
    };

    info!(
        "Rendering image layer {:?} entity {:?} - transform: {:?}, size: {:?}x{:?}",
        name, event.entity, transform.translation, image_data.width, image_data.height
    );

    // Calculate scale if custom dimensions are specified
    let scale = if let (Some(width), Some(height)) = (image_data.width, image_data.height) {
        if let Some(image) = images.get(&image_data.image_handle) {
            let image_size = image.size_f32();
            Vec3::new(width / image_size.x, height / image_size.y, 1.0)
        } else {
            Vec3::ONE
        }
    } else {
        Vec3::ONE
    };

    // Adjust Y position using MapGeometry pattern: bevy_y = map_height - tiled_y
    // The layer transform currently has Y = -offset_y (relative coords)
    // We need Y = map_pixel_height + (-offset_y) = map_pixel_height - offset_y
    let adjusted_y = image_data.map_pixel_height + transform.translation.y;

    // Insert sprite component with adjusted transform
    // BottomLeft anchor means images extend up and right from their position
    commands.entity(event.entity).insert((
        Sprite {
            image: image_data.image_handle.clone(),
            color: image_data.tint_color.unwrap_or(Color::WHITE),
            ..default()
        },
        Anchor(Vec2::new(-0.5, -0.5)), // BottomLeft - images extend up and right
        Transform {
            translation: Vec3::new(transform.translation.x, adjusted_y, transform.translation.z),
            rotation: transform.rotation,
            scale,
        },
    ));

    info!("Created sprite for image layer at adjusted Y={}", adjusted_y);
}
