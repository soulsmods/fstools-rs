use std::io::Cursor;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, LoadContext},
    prelude::*,
};
use fstools_formats::tpf::TPF;
use thiserror::Error;

use crate::types::binder::{Archive, ArchiveEntry};

#[derive(Asset, Deref, TypePath, Debug)]
pub struct TPFAsset(TPF);

#[derive(Debug, Error)]
pub enum TPFAssetLoaderError {
    #[error("Could not load tpf: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not load tpf texture: {0}")]
    TextureParse(#[from] TextureError),
}

#[derive(TypePath)]
pub struct TpfAssetLoader;

impl AssetLoader for TpfAssetLoader {
    type Asset = Archive;
    type Settings = ();
    type Error = TPFAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;

        let mut archive = Archive::default();
        let mut cursor = Cursor::new(&buffer);

        let tpf = TPF::from_reader(&mut cursor)?;
        for texture in tpf.textures.iter() {
            let handle = load_context.labeled_asset_scope(texture.name.clone(), |_| {
                let bytes = texture.bytes(&mut cursor)?;

                Ok::<_, std::io::Error>(ArchiveEntry { data: bytes })
            })?;

            archive
                .files
                .insert(format!("{}.dds", &texture.name), handle);
        }

        Ok(archive)
    }

    fn extensions(&self) -> &[&str] {
        &["tpf", "tpf.dcx"]
    }
}
