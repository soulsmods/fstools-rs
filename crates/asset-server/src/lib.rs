use std::{io, io::Read, path::PathBuf, pin::Pin, sync::Arc, task::Poll};

use bevy::{
    app::{App, Plugin},
    asset::{
        io::{AssetSource, AssetSourceId},
        AssetApp,
    },
    prelude::{Deref, DerefMut},
};
use fstools_dvdbnd::{ArchiveKeyProvider, DvdBnd};
use futures_lite::AsyncRead;

use crate::{
    dvdbnd::DvdBndAssetSource,
    types::{
        bnd4::{Archive, Bnd4Loader},
        flver::{FlverAsset, FlverLoader},
        part::{PartsArchiveLoader, PartsAsset},
    },
    vfs::{Vfs, VfsAssetSource},
};
use crate::types::bnd4::ArchiveEntry;

mod dvdbnd;
pub mod types;
pub mod vfs;

pub struct FsAssetSourcePlugin {
    dvd_bnd: Arc<DvdBnd>,
}

impl FsAssetSourcePlugin {
    pub fn new(
        data_archives: &[PathBuf],
        key_provider: impl ArchiveKeyProvider,
    ) -> std::io::Result<Self> {
        let dvd_bnd = Arc::new(DvdBnd::create(data_archives, &key_provider)?);

        Ok(Self { dvd_bnd })
    }
}

impl Plugin for FsAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        let dvd_bnd = self.dvd_bnd.clone();
        let vfs = Vfs::default();

        app.insert_resource(vfs.clone());
        app.register_asset_source(
            AssetSourceId::from("dvdbnd"),
            AssetSource::build().with_reader(move || Box::new(DvdBndAssetSource(dvd_bnd.clone()))),
        );

        app.register_asset_source(
            AssetSourceId::from("vfs"),
            AssetSource::build().with_reader(move || Box::new(VfsAssetSource(vfs.clone()))),
        );
    }
}

pub struct FsFormatsPlugin;

impl Plugin for FsFormatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FlverAsset>()
            .init_asset::<PartsAsset>()
            .init_asset::<Archive>()
            .init_asset::<ArchiveEntry>()
            .register_asset_loader(FlverLoader)
            .register_asset_loader(PartsArchiveLoader)
            .register_asset_loader(Bnd4Loader);
    }
}

#[derive(Deref, DerefMut)]
struct SimpleReader<R: Read>(R);

impl<R: Read> Unpin for SimpleReader<R> {}

impl<R: Read> AsyncRead for SimpleReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let reader = self.get_mut();
        Poll::Ready(reader.read(buf))
    }
}
