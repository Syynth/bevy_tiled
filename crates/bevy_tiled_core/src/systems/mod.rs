//! Systems for entity spawning and management.

pub mod context;
pub mod spawn;

pub use context::SpawnContext;
pub use spawn::process_loaded_maps;
