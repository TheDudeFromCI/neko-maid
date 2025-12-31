//! The NekoMaid style asset, and asset loader for NekoMaid ui files.

use std::time::Instant;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError};
use bevy::prelude::*;

use crate::native::NATIVE_WIDGETS;
use crate::parse::module::Module;
use crate::parse::{NekoMaidParseError, NekoMaidParser};

/// A NekoMaid UI asset.
#[derive(Debug, Asset, TypePath, Deref)]
pub struct NekoMaidUI(Module);

/// The asset loader for NekoMaid ui files.
#[derive(Debug, Default)]
pub struct NekoMaidAssetLoader;
impl AssetLoader for NekoMaidAssetLoader {
    type Asset = NekoMaidUI;
    type Settings = ();
    type Error = NekoMaidAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let now = Instant::now();

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let text_file = String::from_utf8(bytes)?;
        let mut parser = NekoMaidParser::tokenize(&text_file)?;

        for native in NATIVE_WIDGETS.iter() {
            parser.register_native_widget(native.clone());
        }

        for import in parser.predict_imports().clone() {
            let path = load_context.asset_path();
            let Ok(module_path) = path.resolve(&format!("../{}.neko_ui", import)) else {
                continue;
            };

            let asset = load_context
                .loader()
                .immediate()
                .load::<NekoMaidUI>(&module_path)
                .await?;

            let module = asset.get().0.clone();
            parser.add_module(import.clone(), module);
        }

        let module = parser.finish()?;

        let elapsed = now.elapsed().as_millis();
        debug!(
            "Loaded NekoMaid UI asset {} in {} ms.",
            load_context.path().display(),
            elapsed,
        );

        Ok(NekoMaidUI(module))
    }

    fn extensions(&self) -> &[&str] {
        &["neko_ui"]
    }
}

/// Errors that can occur while loading a NekoMaid asset.
#[derive(Debug, thiserror::Error)]
pub enum NekoMaidAssetLoaderError {
    /// An I/O error occurred while loading the asset.
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// The asset contained invalid UTF-8.
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    /// An error occurred while parsing the asset.
    #[error("Syntax error: {0}")]
    FailedToParse(#[from] NekoMaidParseError),

    /// An error occurred while loading a dependency.
    #[error("{0}")]
    FailedToLoadDependency(#[from] LoadDirectError),
}
