//! Property handling and component registration for Tiled custom properties.
//!
//! This module provides:
//! - Type registry for `#[derive(TiledClass)]` components
//! - JSON export for Tiled editor integration
//! - Property deserialization (Phase 2)
//! - Merged property data (Phase 4)

use bevy::prelude::*;

pub mod deserialize;
pub mod export;
pub mod registry;

pub use deserialize::FromTiledProperty;
pub use export::export_types_to_json;
pub use registry::{TiledClassInfo, TiledClassRegistry};

/// Placeholder for `MergedProperties` component.
///
/// Will be properly implemented in Phase 4.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MergedProperties {
    // Phase 4: actual implementation
}
