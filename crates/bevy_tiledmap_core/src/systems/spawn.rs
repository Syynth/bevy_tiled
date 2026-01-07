//! Main reactive spawning system.

use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;
use bevy_tiledmap_assets::prelude::{TiledMapAsset, TiledTilesetAsset, TiledWorldAsset};

use crate::components::{MapsInWorld, TiledMap, TiledWorld, TiledWorldOf};
use crate::events::{MapSpawned, WorldSpawned};
use crate::plugin::LayerZConfig;
use crate::spawn::spawn_map;
use crate::systems::SpawnContext;

/// Marker component to trigger map respawning.
///
/// Add this component to force the map to be respawned even if it hasn't changed.
#[derive(Component)]
pub struct RespawnTiledMap;

/// Reactive system that detects when `TiledMapAsset` loading completes and spawns entities.
///
/// Runs in `PreUpdate` before user systems.
///
/// # Triggers
///
/// - `Changed<TiledMap>` - When map handle is added or changed
/// - `With<RespawnTiledMap>` - When manual respawn is requested
///
/// # Use Cases
///
/// 1. **Initial spawn**: User creates entity with `TiledMap` component
/// 2. **Hot reload**: Asset changes, system detects and respawns
/// 3. **Dynamic loading**: Runtime map loading for procedural levels
pub fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    map_assets: Res<Assets<TiledMapAsset>>,
    tileset_assets: Res<Assets<TiledTilesetAsset>>,
    template_assets: Res<Assets<bevy_tiledmap_assets::prelude::TiledTemplateAsset>>,
    registry: Res<crate::properties::TiledClassRegistry>,
    type_registry: Res<AppTypeRegistry>,
    z_config: Res<LayerZConfig>,
    mut commands: Commands,
    mut map_query: Query<(Entity, &TiledMap), Or<(Without<crate::components::LayersInMap>, With<RespawnTiledMap>)>>,
) {
    for (map_entity, tiled_map) in map_query.iter_mut() {
        info!("Processing map entity {:?}", map_entity);

        // Check if all dependencies have finished loading
        let load_state = asset_server.get_recursive_dependency_load_state(&tiled_map.handle);
        info!("Load state for map entity {:?}: {:?}", map_entity, load_state);

        let Some(RecursiveDependencyLoadState::Loaded) = load_state
        else {
            continue;
        };

        info!("Map dependencies fully loaded, getting map asset");

        // Get the map asset
        let Some(map_asset) = map_assets.get(&tiled_map.handle) else {
            // Asset handle loaded but asset not found
            warn!("Map asset loaded but not found in Assets resource!");
            continue;
        };

        // Get map name from asset path (only if entity doesn't already have a name)
        let map_name = asset_server
            .get_path(&tiled_map.handle)
            .map(|p| {
                p.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Map")
                    .to_string()
            })
            .unwrap_or_else(|| "Map".to_string());

        info!("Spawning map hierarchy for '{}'", map_name);

        // Add name to map entity if it doesn't have one
        commands.entity(map_entity).insert(Name::new(format!("Map: {}", map_name)));

        // Create spawn context with asset references
        let context = SpawnContext::new(map_asset, &tileset_assets, &template_assets, &registry, &asset_server);

        // Spawn the map hierarchy
        spawn_map(&mut commands, map_entity, &context, &type_registry, &z_config);

        info!("Map hierarchy spawned successfully");

        // Trigger MapSpawned event on the entity for observers
        commands.entity(map_entity).trigger(|entity| MapSpawned { entity });

        // Remove RespawnTiledMap marker if present
        commands.entity(map_entity).remove::<RespawnTiledMap>();
    }
}

/// Marker component to trigger world respawning.
///
/// Add this component to force the world to be respawned even if it hasn't changed.
#[derive(Component)]
pub struct RespawnTiledWorld;

/// Reactive system that detects when `TiledWorldAsset` loading completes and spawns map entities.
///
/// Runs in `PreUpdate` before user systems.
///
/// # Triggers
///
/// - `Changed<TiledWorld>` - When world handle is added or changed
/// - `With<RespawnTiledWorld>` - When manual respawn is requested
///
/// # Behavior
///
/// For each map in the world, spawns a child entity with `TiledMap` component.
/// Maps are positioned according to their coordinates in the `.world` file.
pub fn process_loaded_worlds(
    asset_server: Res<AssetServer>,
    world_assets: Res<Assets<TiledWorldAsset>>,
    _map_assets: Res<Assets<TiledMapAsset>>,
    mut commands: Commands,
    mut world_query: Query<
        (Entity, &TiledWorld),
        Or<(Without<MapsInWorld>, With<RespawnTiledWorld>)>,
    >,
) {
    for (world_entity, tiled_world) in world_query.iter_mut() {
        info!("Processing world entity {:?}", world_entity);

        // Check if all dependencies have finished loading
        let load_state = asset_server.get_recursive_dependency_load_state(&tiled_world.handle);
        info!(
            "Load state for world entity {:?}: {:?}",
            world_entity, load_state
        );

        let Some(RecursiveDependencyLoadState::Loaded) = load_state else {
            continue;
        };

        info!("World dependencies fully loaded, getting world asset");

        // Get the world asset
        let Some(world_asset) = world_assets.get(&tiled_world.handle) else {
            warn!("World asset loaded but not found in Assets resource!");
            continue;
        };

        // Get world name from asset path
        let world_name = asset_server
            .get_path(&tiled_world.handle)
            .map(|p| {
                p.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("World")
                    .to_string()
            })
            .unwrap_or_else(|| "World".to_string());

        // Add name to world entity
        commands.entity(world_entity).insert(Name::new(world_name.clone()));

        info!(
            "World '{}' found with {} maps, spawning map entities",
            world_name,
            world_asset.map_count()
        );

        // Track spawned map entities for the MapsInWorld component
        let mut map_entities = Vec::new();

        // Spawn a TiledMap entity for each map in the world
        for world_map in &world_asset.world.maps {
            // Get the map handle from the world asset
            let Some(map_handle) = world_asset.maps.get(&world_map.filename) else {
                warn!(
                    "Map '{}' referenced in world but not loaded",
                    world_map.filename
                );
                continue;
            };

            // Get map name from filename (without extension)
            let map_name = std::path::Path::new(&world_map.filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&world_map.filename)
                .to_string();

            // Calculate the position from the world map coordinates
            // Tiled uses top-left origin, so we need to adjust Y coordinate
            let position = Vec3::new(world_map.x as f32, -(world_map.y as f32), 0.0);

            info!(
                "Spawning map '{}' at position {:?}",
                map_name, position
            );

            // Spawn the map entity as a child of the world
            let map_entity = commands
                .spawn((
                    Name::new(format!("Map: {}", map_name)),
                    TiledMap {
                        handle: map_handle.clone(),
                    },
                    Transform::from_translation(position),
                    TiledWorldOf(world_entity),
                ))
                .id();

            commands.entity(world_entity).add_child(map_entity);
            map_entities.push(map_entity);
        }

        // Add MapsInWorld component to track the spawned maps
        // Also add PendingWorldSpawn to track when all maps finish processing
        commands
            .entity(world_entity)
            .insert((MapsInWorld(map_entities.clone()), PendingWorldSpawn(map_entities)));

        // Remove RespawnTiledWorld marker if present
        commands.entity(world_entity).remove::<RespawnTiledWorld>();

        info!("World hierarchy spawned successfully");
    }
}

/// Marker component to track worlds waiting for all maps to finish spawning.
#[derive(Component)]
pub struct PendingWorldSpawn(pub Vec<Entity>);

/// System that checks if all maps in a world have finished spawning.
/// Fires `WorldSpawned` when all child maps have `LayersInMap`.
pub fn check_world_spawn_complete(
    mut commands: Commands,
    world_query: Query<(Entity, &PendingWorldSpawn)>,
    map_query: Query<&crate::components::LayersInMap>,
) {
    for (world_entity, pending) in &world_query {
        // Check if all maps have LayersInMap (indicating spawn complete)
        let all_maps_ready = pending.0.iter().all(|&map_entity| {
            map_query.get(map_entity).is_ok()
        });

        if all_maps_ready {
            info!("All maps in world {:?} are ready, firing WorldSpawned", world_entity);
            commands.entity(world_entity).trigger(|entity| WorldSpawned { entity });
            commands.entity(world_entity).remove::<PendingWorldSpawn>();
        }
    }
}
