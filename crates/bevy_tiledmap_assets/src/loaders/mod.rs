use bevy::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use tiled::DefaultResourceCache;

pub mod map;
pub mod template;
pub mod tileset;
pub mod world;

/// Shared cache for `tiled::Loader` to prevent duplicate file parsing
///
/// The `tiled` crate's `Loader` uses a resource cache to avoid re-parsing the same
/// `.tsx` or `.tx` file multiple times when referenced by multiple maps/templates.
///
/// We wrap it in `Arc<Mutex<>>` so all `AssetLoader` instances can share the same
/// cache across asset loading operations, even across threads.
///
/// This is critical for performance: without a shared cache, each map would
/// independently parse all its referenced tilesets and templates, causing
/// unnecessary file I/O and parsing overhead.
#[derive(Resource, Clone, Default, Debug)]
pub struct TiledResourceCache(pub Arc<Mutex<DefaultResourceCache>>);
