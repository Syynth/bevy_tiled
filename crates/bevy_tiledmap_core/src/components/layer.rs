//! Layer components.

use bevy::prelude::*;

/// Layer type marker component.
///
/// Attached to layer entities to indicate what type of layer they represent.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub enum TiledLayer {
    /// Tile layer - will have `TileLayerData` component
    Tiles,
    /// Object layer - will have object entities as children
    Objects,
    /// Image layer - will have `ImageLayerData` component
    Image,
    /// Group layer - hierarchical container, can have other layers as children
    Group,
}

/// Tiled's original layer ID.
///
/// Useful for looking up layer-specific data (like properties) from the `TiledMapAsset`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct LayerId(pub u32);

/// Image layer data component.
///
/// Attached to image layer entities. Layer 3 rendering plugins add Sprite components.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ImageLayerData {
    /// Handle to the image asset
    pub image_handle: Handle<Image>,

    /// Image width (if specified in Tiled, otherwise use image dimensions)
    pub width: Option<f32>,

    /// Image height (if specified in Tiled, otherwise use image dimensions)
    pub height: Option<f32>,

    /// Tint color for the image layer (from Tiled's tintcolor attribute)
    pub tint_color: Option<Color>,

    /// Map pixel height for coordinate conversion in Layer 3 rendering.
    /// Used to position images correctly in Bevy's Y-up coordinate system.
    pub map_pixel_height: f32,
}
