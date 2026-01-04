//! Shape conversion utilities for Tiled objects to `Avian2D` colliders.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tiled_assets::prelude::TiledTilesetAsset;
use bevy_tiled_core::components::object::TiledObject;

/// Convert a `TiledObject` to an `Avian2D` collider.
///
/// # Returns
///
/// - `Some(Collider)` if the object shape can be converted to a collider
/// - `None` if the object type doesn't support colliders (e.g., Text objects)
///
/// # Supported Shapes
///
/// | Tiled Shape | Avian Collider |
/// |-------------|----------------|
/// | Rectangle | `Collider::rectangle(width, height)` |
/// | Ellipse | `Collider::circle(radius)` (approximation) |
/// | Polygon | `Collider::convex_hull(vertices)` or `Collider::triangle_mesh()` |
/// | Polyline | `Collider::polyline(vertices, None)` |
/// | Point | `Collider::circle(1.0)` (small sensor) |
/// | Tile | Fallback to rectangle (tileset shapes in Phase 4) |
/// | Text | `None` (no collider) |
pub fn object_to_collider(object: &TiledObject) -> Option<Collider> {
    match object {
        TiledObject::Rectangle { width, height } => {
            Some(Collider::rectangle(*width, *height))
        }

        TiledObject::Ellipse { width, height } => {
            // Use the maximum dimension as diameter for the circle
            // This ensures the circle fully contains the ellipse bounds
            let radius = width.max(*height) / 2.0;
            Some(Collider::circle(radius))
        }

        TiledObject::Polygon { vertices } => {
            // Try convex hull first (more performant)
            if let Some(collider) = Collider::convex_hull(vertices.clone()) {
                Some(collider)
            } else {
                // Fall back to triangle mesh for concave polygons
                warn!(
                    "Failed to create convex hull for polygon, using triangle mesh (less performant)"
                );
                Some(polygon_to_trimesh(vertices))
            }
        }

        TiledObject::Polyline { vertices } => {
            // Polylines don't form closed shapes, so we use Avian's polyline collider
            // The `None` parameter means no joints are rounded
            Some(Collider::polyline(vertices.clone(), None))
        }

        TiledObject::Point => {
            // Point objects become small circle sensors (1.0 radius)
            // Typically used for spawn points, triggers, etc.
            Some(Collider::circle(1.0))
        }

        TiledObject::Tile {
            width,
            height,
            ..
        } => {
            // Phase 1: Use object bounds as rectangle collider
            // Phase 4: Extract collision shape from tileset
            Some(Collider::rectangle(*width, *height))
        }

        TiledObject::Text { .. } => {
            // Text objects don't have physics colliders
            None
        }
    }
}

/// Convert a polygon to a triangle mesh collider.
///
/// This is used as a fallback when a polygon is concave and can't be represented
/// as a convex hull.
///
/// # Implementation Note
///
/// Currently uses a simple ear clipping triangulation. For complex polygons,
/// consider using a more robust triangulation library like `earcutr` or `lyon`.
fn polygon_to_trimesh(vertices: &[Vec2]) -> Collider {
    // Simple triangulation: fan from first vertex
    // This works for simple concave polygons but may not be robust for complex shapes
    let mut indices = Vec::new();

    if vertices.len() < 3 {
        warn!("Polygon has fewer than 3 vertices, creating degenerate triangle");
        return Collider::triangle(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO);
    }

    // Create triangle fan from vertex 0
    for i in 1..vertices.len() - 1 {
        indices.push([0u32, i as u32, (i + 1) as u32]);
    }

    Collider::trimesh(vertices.to_vec(), indices)
}

/// Get collision shape from a tileset tile.
///
/// Extracts collision shape data defined in the tileset editor for a specific tile.
/// If the tile has multiple collision objects, they are combined into a compound collider.
///
/// # Arguments
///
/// * `tileset` - The tileset asset containing the tile
/// * `local_tile_id` - The local tile ID (0-based, NOT a GID)
///
/// # Returns
///
/// - `Some(Collider)` if the tile has collision shapes defined
/// - `None` if the tile has no collision shapes
pub fn get_tile_collision_shape(
    tileset: &TiledTilesetAsset,
    local_tile_id: u32,
) -> Option<Collider> {
    // Get the tile data from the tileset
    let tile = tileset.tileset.get_tile(local_tile_id)?;

    // Get the collision object group
    let collision_group = tile.collision.as_ref()?;

    // Convert each collision object to a collider
    let mut colliders: Vec<(Vec2, f32, Collider)> = Vec::new();

    for object in collision_group.object_data() {
        // Convert tiled::ObjectShape to Collider
        let collider = match &object.shape {
            tiled::ObjectShape::Rect { width, height } => {
                Collider::rectangle(*width, *height)
            }

            tiled::ObjectShape::Ellipse { width, height } => {
                let radius = width.max(*height) / 2.0;
                Collider::circle(radius)
            }

            tiled::ObjectShape::Polygon { points } => {
                let vertices: Vec<Vec2> = points
                    .iter()
                    .map(|(x, y)| Vec2::new(*x, *y))
                    .collect();

                if let Some(convex) = Collider::convex_hull(vertices.clone()) {
                    convex
                } else {
                    polygon_to_trimesh(&vertices)
                }
            }

            tiled::ObjectShape::Polyline { points } => {
                let vertices: Vec<Vec2> = points
                    .iter()
                    .map(|(x, y)| Vec2::new(*x, *y))
                    .collect();
                Collider::polyline(vertices, None)
            }

            tiled::ObjectShape::Point(x, y) => {
                // Point shapes become small circles at the given position
                colliders.push((Vec2::new(*x, *y), 0.0, Collider::circle(1.0)));
                continue;
            }

            tiled::ObjectShape::Text { .. } => {
                // Text objects don't have colliders
                continue;
            }
        };

        // Add collider with its offset position and rotation
        let position = Vec2::new(object.x, object.y);
        let rotation = object.rotation.to_radians();
        colliders.push((position, rotation, collider));
    }

    // Return the collider(s)
    match colliders.len() {
        0 => None,
        1 => {
            // Single collider - return it directly (ignoring offset for now)
            let (_, _, collider) = colliders.into_iter().next().unwrap();
            Some(collider)
        }
        _ => {
            // Multiple colliders - create a compound
            Some(Collider::compound(colliders))
        }
    }
}

/// Check if a tile has collision shapes defined.
///
/// This is a faster check than `get_tile_collision_shape` when you only need
/// to know if a tile has collision data.
///
/// # Arguments
///
/// * `tileset` - The tileset asset containing the tile
/// * `local_tile_id` - The local tile ID (0-based, NOT a GID)
///
/// # Returns
///
/// `true` if the tile has collision shapes, `false` otherwise
pub fn tile_has_collision_shape(tileset: &TiledTilesetAsset, local_tile_id: u32) -> bool {
    if let Some(tile) = tileset.tileset.get_tile(local_tile_id) {
        if let Some(collision) = &tile.collision {
            return !collision.object_data().is_empty();
        }
    }
    false
}

/// Check if a tile's collision shape is a simple rectangle.
///
/// Returns the size if the tile has exactly one rectangular collision shape.
/// This is used to determine if tiles can be merged during compound collider generation.
///
/// # Arguments
///
/// * `tileset` - The tileset asset containing the tile
/// * `local_tile_id` - The local tile ID (0-based, NOT a GID)
///
/// # Returns
///
/// `Some((width, height))` if the tile has a single rectangular collision shape,
/// `None` otherwise
pub fn get_tile_rectangle_collision_size(
    tileset: &TiledTilesetAsset,
    local_tile_id: u32,
) -> Option<(f32, f32)> {
    let tile = tileset.tileset.get_tile(local_tile_id)?;
    let collision_group = tile.collision.as_ref()?;

    // Only return size if there's exactly one collision object and it's a rectangle
    let objects = collision_group.object_data();
    if objects.len() != 1 {
        return None;
    }

    match objects[0].shape {
        tiled::ObjectShape::Rect { width, height } => Some((width, height)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_to_collider() {
        let object = TiledObject::Rectangle {
            width: 32.0,
            height: 16.0,
        };
        let collider = object_to_collider(&object);
        assert!(collider.is_some());
    }

    #[test]
    fn test_ellipse_to_collider() {
        let object = TiledObject::Ellipse {
            width: 32.0,
            height: 16.0,
        };
        let collider = object_to_collider(&object);
        assert!(collider.is_some());
    }

    #[test]
    fn test_polygon_to_collider() {
        let object = TiledObject::Polygon {
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(10.0, 10.0),
                Vec2::new(0.0, 10.0),
            ],
        };
        let collider = object_to_collider(&object);
        assert!(collider.is_some());
    }

    #[test]
    fn test_polyline_to_collider() {
        let object = TiledObject::Polyline {
            vertices: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)],
        };
        let collider = object_to_collider(&object);
        assert!(collider.is_some());
    }

    #[test]
    fn test_point_to_collider() {
        let object = TiledObject::Point;
        let collider = object_to_collider(&object);
        assert!(collider.is_some());
    }

    #[test]
    fn test_text_no_collider() {
        let object = TiledObject::Text {};
        let collider = object_to_collider(&object);
        assert!(collider.is_none());
    }
}
