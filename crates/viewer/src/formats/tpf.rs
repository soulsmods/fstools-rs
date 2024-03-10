use std::io::Cursor;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        texture::{
            CompressedImageFormats, ImageAddressMode, ImageFormat, ImageSampler,
            ImageSamplerDescriptor, ImageType, TextureError,
        },
    },
    utils::BoxedFuture,
};
use fstools_formats::tpf::TPF;
use fstools_vfs::undo_container_compression;
use thiserror::Error;

use crate::formats::TpfPlugin;

#[derive(Asset, Deref, TypePath, Debug)]
pub struct TPFAsset(TPF);

#[derive(Debug, Error)]
pub enum TPFAssetLoaderError {
    #[error("Could not load tpf: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not load tpf texture: {0}")]
    TextureParse(#[from] TextureError),
}

#[derive(Default)]
pub struct TPFAssetLoader;

impl AssetLoader for TPFAssetLoader {
    type Asset = TPFAsset;
    type Settings = ();
    type Error = TPFAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            // Account for DCX compression
            let decompressed = undo_container_compression(&buffer).unwrap();
            let mut cursor = Cursor::new(&decompressed);

            let tpf = TPF::from_reader(&mut cursor)?;
            for texture in tpf.textures.iter() {
                let bytes = texture.bytes(&mut cursor)?;

                load_context.labeled_asset_scope(texture.name.clone(), |_| {
                    Image::from_buffer(
                        #[cfg(debug_assertions)]
                        texture.name.clone(),
                        &bytes,
                        ImageType::Format(ImageFormat::Dds),
                        CompressedImageFormats::BC,
                        false,
                        ImageSampler::Descriptor(ImageSamplerDescriptor {
                            label: Some(texture.name.clone()),
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            ..Default::default()
                        }),
                        RenderAssetUsages::MAIN_WORLD,
                    )
                    .unwrap()
                });
            }

            Ok(TPFAsset(tpf))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tpf", "tpf.dcx"]
    }
}

impl Plugin for TpfPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TPFAsset>()
            .init_asset_loader::<TPFAssetLoader>();
    }
}
