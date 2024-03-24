use std::{io, path::PathBuf, sync::Arc};

use bevy::{
    app::{App, Plugin},
    asset::io::{AssetSource, AssetSourceId},
    prelude::AssetApp,
};
use fstools_dvdbnd::{ArchiveKeyProvider, DvdBnd};

use crate::asset_source::{
    dvdbnd::DvdBndAssetSource,
    vfs::{watcher::VfsWatcher, Vfs, VfsAssetSource},
};

pub mod dvdbnd;
pub(crate) mod fast_path;
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
