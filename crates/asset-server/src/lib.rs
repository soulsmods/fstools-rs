use std::{path::PathBuf, sync::Arc};

use bevy_app::{App, Plugin};
use bevy_asset::{
    io::{file::FileAssetReader, AssetSource, AssetSourceId},
    AssetApp,
};
use fstools_dvdbnd::{ArchiveKeyProvider, DvdBnd};

use crate::dvdbnd::DvdBndAssetSource;

mod dvdbnd;
mod archive;

pub struct FsAssetsPlugin {
    dvd_bnd: Arc<DvdBnd>,
    local_data_path: Option<PathBuf>,
}

impl FsAssetsPlugin {
    pub fn new(
        data_archives: &[PathBuf],
        key_provider: impl ArchiveKeyProvider,
        local_data_path: Option<PathBuf>,
    ) -> std::io::Result<Self> {
        let dvd_bnd = Arc::new(DvdBnd::create(data_archives, &key_provider)?);

        Ok(Self {
            dvd_bnd,
            local_data_path,
        })
    }
}

impl Plugin for FsAssetsPlugin {
    fn build(&self, app: &mut App) {
        let dvd_bnd = self.dvd_bnd.clone();

        app.register_asset_source(
            AssetSourceId::from("game_dvdbnd"),
            AssetSource::build().with_reader(move || Box::new(DvdBndAssetSource(dvd_bnd.clone()))),
        );

        if let Some(path) = self.local_data_path.clone() {
            app.register_asset_source(
                AssetSourceId::from("game_local_files"),
                AssetSource::build()
                    .with_reader(move || Box::new(FileAssetReader::new(path.clone()))),
            );
        }
    }
}

