//! Tile layer rendering module.

pub mod animations;
pub mod render;
pub mod tilemap_builder;

pub use animations::{update_tile_animations, AnimationFrame, TileAnimation};
pub use render::on_tile_layer_spawned;
pub use tilemap_builder::{TilemapBuilder, TilesetReference};
