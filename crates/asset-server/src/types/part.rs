use std::io::Cursor;

use bevy::{
    asset::{io::Reader, ron, AssetLoader, BoxedFuture, LoadContext},
    prelude::*,
    scene::serde::SceneDeserializer,
};
use fstools_formats::bnd4::BND4;
use futures_lite::AsyncReadExt;

use crate::types::flver::FlverAsset;

#[derive(Asset, TypePath, Debug)]
pub struct PartsAsset {
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}

#[derive(Default)]
pub struct PartsArchiveLoader;

impl AssetLoader for PartsArchiveLoader {
    type Asset = FlverAsset;
    type Settings = ();
    type Error = std::io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let bnd = BND4::from_reader(&mut Cursor::new(&bytes[..]))?;
            let flver = bnd
                .files
                .iter()
                .find(|file| file.path.ends_with("flver"))
                .expect("no_flver");
            //
            // let tpf = bnd
            //     .files
            //     .iter()
            //     .find(|file| file.path.ends_with("tpf"))
            //     .expect("no_tpf");

            let mut flver_ctx = load_context.begin_labeled_asset();
            let flver_asset = flver_ctx
                .load_direct_with_reader(
                    &mut futures_lite::io::Cursor::new(&bnd.data[flver.data_offset as usize..]),
                    "parts/am_m_1100.flver",
                )
                .await
                .unwrap()
                .take::<FlverAsset>()
                .expect("not flver");

            Ok(flver_asset)
        })
    }
}
