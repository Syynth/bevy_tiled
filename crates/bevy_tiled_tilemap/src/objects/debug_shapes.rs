//! Debug rendering for object shapes using gizmos.

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_tiled_core::components::object::TiledObject;

/// System that renders object shapes as gizmos for debugging.
///
/// Draws different shapes with different colors:
/// - Rectangle: Green
/// - Ellipse: Blue
/// - Polygon: Yellow
/// - Polyline: Cyan
/// - Point: Red
/// - Tile: Magenta (bounding box)
pub fn render_object_shapes(
    mut gizmos: Gizmos,
    objects: Query<(&TiledObject, &GlobalTransform)>,
) {
    for (object, transform) in &objects {
        let position = transform.translation().truncate();

        match object {
            TiledObject::Point => {
                // Draw a small cross for points
                let size = 5.0;
                gizmos.line_2d(
                    position + Vec2::new(-size, 0.0),
                    position + Vec2::new(size, 0.0),
                    css::RED,
                );
                gizmos.line_2d(
                    position + Vec2::new(0.0, -size),
                    position + Vec2::new(0.0, size),
                    css::RED,
                );
            }

            TiledObject::Rectangle { width, height } => {
                // Draw rectangle outline
                gizmos.rect_2d(
                    position + Vec2::new(*width / 2.0, *height / 2.0),
                    0.0,
                    Vec2::new(*width, *height),
                    css::GREEN,
                );
            }

            TiledObject::Ellipse { width, height } => {
                // Draw ellipse as circle (Bevy doesn't have ellipse gizmo yet)
                // Use average of width/height as radius
                let radius = (*width + *height) / 4.0;
                gizmos.circle_2d(
                    position + Vec2::new(*width / 2.0, *height / 2.0),
                    radius,
                    css::BLUE,
                );
            }

            TiledObject::Polygon { vertices } => {
                // Draw polygon outline
                if vertices.len() >= 2 {
                    for i in 0..vertices.len() {
                        let next = (i + 1) % vertices.len();
                        gizmos.line_2d(
                            position + vertices[i],
                            position + vertices[next],
                            css::YELLOW,
                        );
                    }
                }
            }

            TiledObject::Polyline { vertices } => {
                // Draw polyline (not closed)
                if vertices.len() >= 2 {
                    for i in 0..vertices.len() - 1 {
                        gizmos.line_2d(
                            position + vertices[i],
                            position + vertices[i + 1],
                            css::CYAN,
                        );
                    }
                }
            }

            TiledObject::Tile {
                width, height, ..
            } => {
                // Draw bounding box for tile objects
                gizmos.rect_2d(
                    position + Vec2::new(*width / 2.0, *height / 2.0),
                    0.0,
                    Vec2::new(*width, *height),
                    css::MAGENTA,
                );
            }

            TiledObject::Text {} => {
                // No debug rendering for text objects yet
            }
        }
    }
}
