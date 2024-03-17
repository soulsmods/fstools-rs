use std::{io, io::Read, path::PathBuf, pin::Pin, sync::Arc, task::Poll};

use bevy::{
    app::{App, Plugin},
    asset::{
        io::{AssetSource, AssetSourceId},
        AssetApp, Handle,
    },
    prelude::{Deref, DerefMut},
};
use fstools_dvdbnd::{ArchiveKeyProvider, DvdBnd};
use futures_lite::AsyncRead;

use crate::types::msb::{MsbAsset, MsbAssetLoader, MsbPartAsset, MsbPointAsset};
use crate::{
    dvdbnd::DvdBndAssetSource,
    types::{
        bnd4::{Archive, ArchiveEntry, Bnd4Loader},
        flver::{FlverAsset, FlverLoader},
    },
    vfs::{watcher::VfsWatcher, Vfs, VfsAssetSource},
};

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

        app.register_asset_source(
            AssetSourceId::from("dvdbnd"),
            AssetSource::build().with_reader(move || Box::new(DvdBndAssetSource(dvd_bnd.clone()))),
        );

        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let vfs = Vfs::new(event_sender);

        app.insert_resource(vfs.clone());
        app.register_asset_source(
            AssetSourceId::from("vfs"),
            AssetSource::build()
                .with_reader(move || Box::new(VfsAssetSource(vfs.clone())))
                .with_watcher(move |sender| {
                    let mut watcher = Box::new(VfsWatcher::new(event_receiver.clone(), sender));
                    watcher.start();

                    Some(watcher)
                }),
        );
    }
}

pub struct FsFormatsPlugin;

impl Plugin for FsFormatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FlverAsset>()
            .register_type::<FlverAsset>()
            .register_type::<Handle<FlverAsset>>()
            .init_asset::<Archive>()
            .init_asset::<ArchiveEntry>()
            .init_asset::<MsbAsset>()
            .register_asset_loader(MsbAssetLoader)
            .register_asset_loader(FlverLoader)
            .register_asset_loader(Bnd4Loader);
        app.init_asset::<MsbAsset>()
            .init_asset::<MsbPointAsset>()
            .init_asset::<MsbPartAsset>()
            .init_asset_loader::<MsbAssetLoader>();
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
