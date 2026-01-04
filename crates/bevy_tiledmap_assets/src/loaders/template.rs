use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};
use thiserror::Error;

use crate::assets::template::TiledTemplateAsset;
use crate::loaders::TiledResourceCache;

/// Asset loader for Tiled templates (.tx files)
///
/// Templates can optionally reference a tileset if the template object is tile-based.
#[derive(Default)]
pub struct TiledTemplateAssetLoader {
    pub cache: TiledResourceCache,
}

#[derive(Debug, Error)]
pub enum TemplateLoaderError {
    #[error("Failed to load template: {0}")]
    TiledError(#[from] tiled::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl AssetLoader for TiledTemplateAssetLoader {
    type Asset = TiledTemplateAsset;
    type Settings = ();
    type Error = TemplateLoaderError;

    fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            // NOTE: The tiled crate v0.15 does not provide a public `load_template()` method.
            // Templates are loaded automatically when referenced by objects in maps and cached
            // internally using ResourceCache.
            //
            // ACCESSING TEMPLATE PROPERTIES:
            // When a map object references a template, you can access template properties via:
            //   - map.map.layers() → object layer → objects → object.template → template.object.properties
            //   - Or use the convenience method: template.properties()
            //
            // FUTURE: If standalone template loading is needed, we'll implement manual XML parsing
            // for .tx files. For now, templates are only accessible via map dependencies.

            let path = load_context.asset_path().path();

            Err(TemplateLoaderError::InvalidPath(format!(
                "Direct template loading is not supported (tiled crate v0.15 limitation). \
                 Templates are loaded automatically when referenced by map objects. \
                 Access template properties via map objects: object.template.properties(). \
                 Path attempted: {:?}",
                path
            )))
        }
    }

    fn extensions(&self) -> &[&str] {
        &["tx"]
    }
}
