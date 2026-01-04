//! Object rendering for Tiled objects.

pub mod tile_objects;

#[cfg(feature = "debug_shapes")]
pub mod debug_shapes;

pub use tile_objects::on_tile_object_spawned;

#[cfg(feature = "debug_shapes")]
pub use debug_shapes::render_object_shapes;
