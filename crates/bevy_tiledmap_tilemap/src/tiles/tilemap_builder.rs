//! Converts `TileLayerData` into `bevy_ecs_tilemap` structures.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_tiledmap_assets::prelude::TiledTilesetAsset;
use bevy_tiledmap_core::components::tile::{TileInstance, TileLayerData};

use super::animations::{AnimationFrame, TileAnimation};

/// Builds `bevy_ecs_tilemap` structures from Layer 2's `TileLayerData`.
///
/// Handles the conversion of pre-processed tile data into performant
/// tilemap rendering structures. For Phase 1, supports single-tileset layers.
/// Multi-tileset support will be added in Phase 2.
pub struct TilemapBuilder;

impl TilemapBuilder {
    /// Build tilemap structures from tile layer data.
    ///
    /// Creates `bevy_ecs_tilemap` entities as children of the layer entity.
    ///
    /// # Arguments
    ///
    /// * `commands` - Command buffer for spawning entities
    /// * `layer_entity` - The layer entity to attach tilemaps to
    /// * `tile_data` - Pre-processed tile data from Layer 2
    /// * `tileset_assets` - Access to tileset assets
    pub fn build(
        commands: &mut Commands,
        layer_entity: Entity,
        tile_data: &TileLayerData,
        tileset_assets: &Assets<TiledTilesetAsset>,
    ) {
        // Group tiles by tileset for multi-tileset support
        let tiles_by_tileset = Self::group_by_tileset(tile_data);

        if tiles_by_tileset.is_empty() {
            info!("Layer has no tiles, skipping tilemap creation");
            return;
        }

        // Create a separate tilemap for each tileset
        for (tileset_handle, tiles) in tiles_by_tileset {
            let Some(tileset) = tileset_assets.get(&tileset_handle) else {
                warn!(
                    "Tileset not loaded yet for handle {:?}, skipping",
                    tileset_handle
                );
                continue;
            };

            Self::create_tilemap(
                commands,
                layer_entity,
                tiles,
                tileset,
                tileset_handle,
                tile_data.width,
                tile_data.height,
            );
        }
    }

    /// Group tiles by their tileset handle.
    ///
    /// This is necessary because `bevy_ecs_tilemap` requires one tilemap per texture.
    /// Layers can use multiple tilesets, so we create separate tilemaps for each.
    fn group_by_tileset(
        tile_data: &TileLayerData,
    ) -> HashMap<Handle<TiledTilesetAsset>, Vec<(u32, u32, TileInstance)>> {
        let mut grouped = HashMap::new();

        for (x, y, tile) in tile_data.iter_tiles() {
            grouped
                .entry(tile.tileset_handle.clone())
                .or_insert_with(Vec::new)
                .push((x, y, tile.clone()));
        }

        grouped
    }

    /// Extract animation data for a specific tile from the tileset.
    ///
    /// Returns None if the tile is not animated.
    #[cfg(feature = "animations")]
    fn get_tile_animation(tileset: &TiledTilesetAsset, tile_id: u32) -> Option<TileAnimation> {
        // Find the tile in the tileset's tile data and extract animation
        tileset
            .tileset
            .tiles()
            .find(|(id, _tile)| *id == tile_id)
            .and_then(|(_id, tile)| {
                tile.animation.as_ref().map(|frames| {
                    let animation_frames: Vec<AnimationFrame> = frames
                        .iter()
                        .map(|frame| AnimationFrame {
                            tile_id: frame.tile_id,
                            duration_ms: frame.duration,
                        })
                        .collect();

                    TileAnimation::new(animation_frames)
                })
            })
    }

    /// Create a single tilemap for a specific tileset.
    fn create_tilemap(
        commands: &mut Commands,
        layer_entity: Entity,
        tiles: Vec<(u32, u32, TileInstance)>,
        tileset: &TiledTilesetAsset,
        tileset_handle: Handle<TiledTilesetAsset>,
        width: u32,
        height: u32,
    ) {
        // Check if this is an image collection or atlas tileset
        if tileset.atlas_image.is_some() {
            // Use bevy_ecs_tilemap for atlas tilesets
            Self::create_atlas_tilemap(
                commands,
                layer_entity,
                tiles,
                tileset,
                tileset_handle,
                width,
                height,
            );
        } else {
            // Use simple sprites for image collection tilesets
            Self::create_image_collection_tilemap(commands, layer_entity, tiles, tileset, height);
        }
    }

    /// Create tilemap using simple sprites for image collection tilesets.
    fn create_image_collection_tilemap(
        commands: &mut Commands,
        layer_entity: Entity,
        tiles: Vec<(u32, u32, TileInstance)>,
        tileset: &TiledTilesetAsset,
        height: u32,
    ) {
        let tile_size = tileset.tile_size;
        let tile_count = tiles.len();

        for (x, y, tile_instance) in tiles {
            // Get the image handle for this specific tile
            let Some(tile_image_handle) = tileset.tile_images.get(&tile_instance.tile_id) else {
                warn!("Tile ID {} not found in tileset", tile_instance.tile_id);
                continue;
            };

            // Calculate local position for this tile relative to the layer
            // Flip Y: Tiled y=0 is top, Bevy y=0 is bottom
            // Use positive Y coordinates to match MapGeometry bounds
            let flipped_y = height - 1 - y;
            let world_x = (x as f32 + 0.5) * tile_size.x as f32;
            let world_y = (flipped_y as f32 + 0.5) * tile_size.y as f32;

            // Spawn a sprite for this tile
            let mut sprite_bundle = Sprite {
                image: tile_image_handle.clone(),
                flip_x: tile_instance.flipped_h,
                flip_y: tile_instance.flipped_v,
                ..default()
            };

            // Handle diagonal flip (requires rotation + flip)
            let mut transform = Transform::from_xyz(world_x, world_y, 0.0);
            if tile_instance.flipped_d {
                // Diagonal flip is a 90° rotation + horizontal flip
                transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
                sprite_bundle.flip_x = !sprite_bundle.flip_x;
            }

            commands.entity(layer_entity).with_children(|parent| {
                parent.spawn((sprite_bundle, transform));
            });
        }

        info!(
            "Created image collection tilemap with {} tiles as sprites",
            tile_count
        );
    }

    /// Create tilemap using `bevy_ecs_tilemap` for atlas tilesets.
    fn create_atlas_tilemap(
        commands: &mut Commands,
        layer_entity: Entity,
        tiles: Vec<(u32, u32, TileInstance)>,
        tileset: &TiledTilesetAsset,
        tileset_handle: Handle<TiledTilesetAsset>,
        width: u32,
        height: u32,
    ) {
        let Some(ref atlas_image) = tileset.atlas_image else {
            warn!("Expected atlas tileset but atlas_image is None");
            return;
        };

        let map_size = TilemapSize {
            x: width,
            y: height,
        };

        let tile_size = TilemapTileSize {
            x: tileset.tile_size.x as f32,
            y: tileset.tile_size.y as f32,
        };

        let grid_size = TilemapGridSize {
            x: tileset.tile_size.x as f32,
            y: tileset.tile_size.y as f32,
        };

        // Create tile storage
        let mut tile_storage = TileStorage::empty(map_size);

        // Spawn empty tilemap entity first (following bevy_ecs_tilemap pattern)
        let tilemap_entity = commands.spawn_empty().id();

        // Make tilemap a child of layer entity
        commands.entity(layer_entity).add_child(tilemap_entity);

        // Collect tile entities to parent them to the tilemap
        let mut tile_entities = Vec::with_capacity(tiles.len());

        // Spawn individual tiles with correct TilemapId
        // Flip Y coordinate: Tiled has (0,0) at top-left, bevy_ecs_tilemap at bottom-left
        for (x, y, tile_instance) in tiles {
            let flipped_y = height - 1 - y;
            let tile_pos = TilePos { x, y: flipped_y };

            let mut entity_commands = commands.spawn(TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(tile_instance.tile_id),
                tilemap_id: TilemapId(tilemap_entity),
                flip: TileFlip {
                    x: tile_instance.flipped_h,
                    y: tile_instance.flipped_v,
                    d: tile_instance.flipped_d,
                },
                ..default()
            });

            // Add animation if this tile is animated
            #[cfg(feature = "animations")]
            if let Some(animation) = Self::get_tile_animation(tileset, tile_instance.tile_id) {
                entity_commands.insert(animation);
            }

            let tile_entity = entity_commands.id();
            tile_storage.set(&tile_pos, tile_entity);
            tile_entities.push(tile_entity);
        }

        // Parent all tile entities to the tilemap for hierarchy organization
        commands.entity(tilemap_entity).add_children(&tile_entities);

        // Now insert the TilemapBundle with populated storage
        // Position tilemap so tiles render with positive Y matching MapGeometry bounds
        // bevy_ecs_tilemap places TilePos y=0 at tilemap origin, so we offset by half tile
        let tilemap_y = tile_size.y / 2.0;
        let texture = TilemapTexture::Single(atlas_image.clone());
        commands.entity(tilemap_entity).insert((
            TilemapBundle {
                grid_size,
                size: map_size,
                storage: tile_storage,
                texture,
                tile_size,
                map_type: TilemapType::Square,
                transform: Transform::from_xyz(tile_size.x / 2.0, tilemap_y, 0.0),
                ..default()
            },
            TilesetReference(tileset_handle),
        ));

        // info!("Created tilemap for tileset with {} tiles, tilemap_y={}, layer_pixel_height={}", tile_count, tilemap_y, layer_pixel_height);
    }
}

/// Component that tracks which tileset a tilemap uses.
///
/// Used for animation lookups and debugging.
#[derive(Component, Debug)]
pub struct TilesetReference(pub Handle<TiledTilesetAsset>);
