//! Debug visualization for Tiled maps.

use bevy::prelude::*;

use crate::components::map::MapGeometry;

/// Resource to enable map geometry debug visualization.
///
/// Insert this resource to enable drawing debug rectangles around each map's geometry.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_tiledmap_core::debug::DebugMapGeometry;
/// fn enable_debug(mut commands: Commands) {
///     commands.insert_resource(DebugMapGeometry::default());
/// }
/// ```
#[derive(Resource, Debug, Clone)]
pub struct DebugMapGeometry {
    /// Color for the map bounds rectangle
    pub bounds_color: Color,
}

impl Default for DebugMapGeometry {
    fn default() -> Self {
        Self {
            bounds_color: Color::srgba(0.0, 1.0, 0.0, 0.8), // Green
        }
    }
}

/// System that draws debug rectangles around each map's geometry.
///
/// Only runs when `DebugMapGeometry` resource is present.
pub fn draw_map_geometry_debug(
    config: Res<DebugMapGeometry>,
    map_query: Query<(&MapGeometry, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    for (geometry, global_transform) in &map_query {
        // Get the world position of the map
        let map_pos = global_transform.translation().truncate();

        // Calculate the corners of the bounds in world space
        let min = map_pos + geometry.bounds.min;
        let max = map_pos + geometry.bounds.max;
        let center = (min + max) / 2.0;
        let size = max - min;

        // Draw the bounds rectangle
        gizmos.rect_2d(
            Isometry2d::from_translation(center),
            size,
            config.bounds_color,
        );

        // Draw corner markers for clarity
        let corner_size = geometry.tile_size.min_element() * 0.5;
        let corners = [
            min,
            Vec2::new(max.x, min.y),
            max,
            Vec2::new(min.x, max.y),
        ];
        for corner in corners {
            gizmos.circle_2d(Isometry2d::from_translation(corner), corner_size, config.bounds_color);
        }
    }
}
