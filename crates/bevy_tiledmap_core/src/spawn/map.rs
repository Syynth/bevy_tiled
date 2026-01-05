//! Map spawning logic.

use bevy::prelude::*;

use crate::components::{LayersInMap, MapGeometry};
use crate::spawn::spawn_layer;
use crate::systems::SpawnContext;

/// Spawn the entity hierarchy for a map.
///
/// Creates layer entities with appropriate components and data:
/// - Tile layers: `TileLayerData` with pre-processed tiles
/// - Object layers: Individual object entities as children
/// - Image layers: `ImageLayerData`
/// - Group layers: Recursive layer hierarchy
///
/// # arguments
///
/// * `commands` - Bevy commands for entity spawning
/// * `map_entity` - The entity with the `TiledMap` component
/// * `context` - Spawn context with asset data access
/// * `type_registry` - App type registry for reflection-based component insertion
pub fn spawn_map(
    commands: &mut Commands,
    map_entity: Entity,
    context: &SpawnContext,
    type_registry: &AppTypeRegistry,
) {
    let mut layer_entities = Vec::new();

    // Spawn each top-level layer (spawn_layer handles recursion for groups)
    for layer in context.map_asset.map.layers() {
        let layer_entity = spawn_layer(commands, &layer, map_entity, context, type_registry);
        layer_entities.push(layer_entity);
    }

    // Create MapGeometry for world-space boundary and coordinate conversion
    let map = &context.map_asset.map;
    let map_geometry = MapGeometry::new(
        map.width,
        map.height,
        map.tile_width as f32,
        map.tile_height as f32,
    );

    // Add components and set up parent-child hierarchy
    commands
        .entity(map_entity)
        .insert((
            LayersInMap(layer_entities.clone()),
            map_geometry,
        ))
        .add_children(&layer_entities);
}
