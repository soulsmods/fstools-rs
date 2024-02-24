use bevy::prelude::*;
use format::flver::FLVER;
use thiserror::Error;
use bevy::utils::BoxedFuture;
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};

#[derive(Asset, Deref, TypePath, Debug)]
pub struct FLVERAsset(FLVER);

#[derive(Debug, Error)]
pub enum FLVERAssetLoaderError {
    #[error("Could not load tpf: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct FLVERAssetLoader;

impl AssetLoader for FLVERAssetLoader {
    type Asset = FLVERAsset;
    type Settings = ();
    type Error = FLVERAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            let mut cursor = std::io::Cursor::new(buffer);
            Ok(FLVERAsset(FLVER::from_reader(&mut cursor)?))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["flver"]
    }
}
