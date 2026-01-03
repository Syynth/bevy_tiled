use bevy::prelude::*;

use crate::assets::tileset::TiledTilesetAsset;

/// Bevy asset wrapper for Tiled templates (.tx files)
///
/// Templates define reusable object configurations in Tiled. They can optionally
/// reference a tileset if the template represents a tile-based object.
///
/// # Template Loading Limitation
///
/// **Important:** Templates are NOT directly loadable as standalone Bevy assets in tiled v0.15.
/// The tiled crate does not provide a public `load_template()` method. Instead, templates are
/// loaded automatically when referenced by objects in maps.
///
/// # Accessing Template Properties
///
/// When a map object uses a template, you can access the template's properties:
///
/// ```rust,ignore
/// // Access via map object layers
/// for layer in map_asset.map.layers() {
///     if let Some(object_layer) = layer.as_object_layer() {
///         for object in object_layer.objects() {
///             if let Some(template) = &object.template {
///                 // Access properties directly
///                 let props = &template.object.properties;
///                 // Or use the convenience method
///                 let props = template.properties();
///             }
///         }
///     }
/// }
/// ```
///
/// # Future Support
///
/// Standalone template loading may be added in the future via manual XML parsing.
#[derive(TypePath, Asset, Debug)]
pub struct TiledTemplateAsset {
    /// Raw Tiled template data (PRESERVE AS-IS)
    ///
    /// All original template data from the .tx file is preserved here.
    /// This includes the object definition and its properties.
    pub template: tiled::Template,

    // ===== ASSET REFERENCES =====
    /// Tileset reference (if the template object uses a tile)
    ///
    /// This is `Some(handle)` when the template's object is tile-based (has a GID).
    /// For non-tile objects (rectangles, polygons, points, etc.), this is `None`.
    pub tileset: Option<Handle<TiledTilesetAsset>>,

    // ===== CUSTOM PROPERTIES =====
    /// Custom properties from the template's object
    ///
    /// NOTE: Templates are not directly loadable as Bevy assets in tiled 0.15.
    /// They are loaded automatically when referenced by map objects.
    /// Access template properties via: `template.object.properties`
    pub properties: crate::properties::Properties,
}

impl TiledTemplateAsset {
    /// Check if this template uses a tile (requires tileset)
    ///
    /// Returns `true` if this template represents a tile-based object,
    /// `false` for geometric objects (rectangle, polygon, point, etc.).
    #[inline]
    pub fn uses_tile(&self) -> bool {
        self.tileset.is_some()
    }

    /// Get the object definition from the template
    ///
    /// Convenience accessor for the object contained in the template.
    #[inline]
    pub fn object(&self) -> &tiled::ObjectData {
        &self.template.object
    }

    /// Get the properties from the template's object
    ///
    /// Convenience accessor for properties. Same as `template.object.properties`.
    #[inline]
    pub fn properties(&self) -> &crate::properties::Properties {
        &self.template.object.properties
    }
}
