//! Sprite rendering for tile objects.

use bevy::prelude::*;
use bevy_tiled_assets::prelude::TiledTilesetAsset;
use bevy_tiled_core::components::object::TiledObject;
use bevy_tiled_core::events::ObjectSpawned;

/// Observer that renders tile objects as sprites.
///
/// When an object with a Tile variant is spawned, this observer:
/// 1. Extracts the texture from the tileset
/// 2. Calculates the texture atlas rectangle (for atlas tilesets)
/// 3. Spawns a Sprite component with the correct texture and size
pub fn on_tile_object_spawned(
    trigger: On<ObjectSpawned>,
    object_query: Query<&TiledObject>,
    mut transform_query: Query<&mut Transform>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok(object) = object_query.get(event.entity) else {
        return;
    };

    // Only handle Tile objects
    let TiledObject::Tile {
        tile_id,
        tileset_handle,
        width,
        height,
    } = object
    else {
        return;
    };

    let Some(tileset) = tileset_assets.get(tileset_handle) else {
        warn!(
            "Tileset not loaded yet for tile object {:?}, skipping sprite creation",
            event.object_id
        );
        return;
    };

    // Get the image for this tile
    let Some(image_handle) = tileset.get_tile_image(*tile_id) else {
        warn!(
            "No image found for tile {} in tileset, skipping",
            tile_id
        );
        return;
    };

    // Calculate scale factor based on object size vs tile size
    let tile_size_vec = Vec2::new(tileset.tile_size.x as f32, tileset.tile_size.y as f32);
    let object_size = Vec2::new(*width, *height);
    let scale = object_size / tile_size_vec;

    // Update the existing Transform's scale (Layer 2 set the position)
    if let Ok(mut transform) = transform_query.get_mut(event.entity) {
        transform.scale = scale.extend(1.0);
    }

    // For image collection tilesets, use the tile's individual image
    if tileset.is_image_collection() {
        commands.entity(event.entity).insert(Sprite {
            image: image_handle.clone(),
            ..default()
        });

        info!(
            "Created sprite for image collection tile object {:?}",
            event.object_id
        );
        return;
    }

    // For texture atlas tilesets, calculate the texture rect
    let texture_rect = calculate_tile_rect(tileset, *tile_id);

    commands.entity(event.entity).insert(Sprite {
        image: image_handle.clone(),
        rect: Some(texture_rect),
        ..default()
    });

    info!(
        "Created sprite for atlas tile object {:?} (tile_id: {})",
        event.object_id, tile_id
    );
}

/// Calculate the texture rectangle for a tile in a texture atlas.
///
/// Takes into account margin, spacing, and grid layout.
fn calculate_tile_rect(tileset: &TiledTilesetAsset, tile_id: u32) -> Rect {
    let columns = tileset.grid_size.x;
    let tile_width = tileset.tile_size.x as f32;
    let tile_height = tileset.tile_size.y as f32;
    let margin = tileset.margin as f32;
    let spacing = tileset.spacing as f32;

    // Calculate column and row
    let col = tile_id % columns;
    let row = tile_id / columns;

    // Calculate position in pixels
    let x = margin + (col as f32 * (tile_width + spacing));
    let y = margin + (row as f32 * (tile_height + spacing));

    Rect {
        min: Vec2::new(x, y),
        max: Vec2::new(x + tile_width, y + tile_height),
    }
}
