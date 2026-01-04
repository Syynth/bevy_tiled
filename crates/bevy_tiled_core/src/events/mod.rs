//! Event system for Layer 3 extension hooks.
//!
//! Events will be properly implemented in Phase 4.

use bevy::prelude::*;

/// Generic event wrapper (placeholder for Phase 4).
#[derive(Event, Debug, Clone)]
pub struct TiledEvent<E> {
    pub entity: Entity,
    pub map_entity: Entity,
    pub event: E,
}

// Event type markers (placeholders for Phase 4)
#[derive(Debug, Clone)]
pub struct MapCreated;

#[derive(Debug, Clone)]
pub struct TileLayerCreated;

#[derive(Debug, Clone)]
pub struct ObjectLayerCreated;

#[derive(Debug, Clone)]
pub struct ImageLayerCreated;

#[derive(Debug, Clone)]
pub struct GroupLayerCreated;

#[derive(Debug, Clone)]
pub struct ObjectCreated;
