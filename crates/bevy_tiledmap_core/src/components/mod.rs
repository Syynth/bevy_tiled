//! Components for the `bevy_tiledmap_core` entity hierarchy.

pub mod layer;
pub mod map;
pub mod object;
pub mod tile;

// Re-export commonly used components
pub use layer::{ImageLayerData, LayerId, TiledLayer};
pub use map::{LayersInMap, ObjectsInMap, TiledLayerMapOf, TiledMap, TiledObjectMapOf};
pub use object::{ObjectId, TiledObject};
pub use tile::{TileInstance, TileLayerData};
