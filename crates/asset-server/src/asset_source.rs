use std::{io, io::Read, path::PathBuf, pin::Pin, sync::Arc, task::Poll};

use bevy::{
    app::{App, Plugin},
    asset::io::{AssetSource, AssetSourceId},
    prelude::{AssetApp, Deref, DerefMut},
};
use fstools_dvdbnd::{ArchiveKeyProvider, DvdBnd};
use futures_lite::AsyncRead;

use crate::asset_source::{
    dvdbnd::DvdBndAssetSource,
    vfs::{watcher::VfsWatcher, Vfs, VfsAssetSource},
};

pub mod dvdbnd;
pub mod vfs;

pub struct FsAssetSourcePlugin {
    dvd_bnd: Arc<DvdBnd>,
}

impl FsAssetSourcePlugin {
    pub fn new(
        data_archives: &[PathBuf],
        key_provider: impl ArchiveKeyProvider,
    ) -> io::Result<Self> {
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
