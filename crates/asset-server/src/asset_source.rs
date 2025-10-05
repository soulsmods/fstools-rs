use std::{io, path::PathBuf, sync::Arc};

use bevy::{
    app::{App, Plugin},
    asset::io::{AssetSourceBuilder, AssetSourceId},
    prelude::AssetApp,
};
use fstools_dvdbnd::{ArchiveKeyProvider, DvdBnd};

use crate::asset_source::dvdbnd::DvdBndAssetSource;

pub mod dvdbnd;
pub mod vfs;

pub struct DvdBndAssetSourcePlugin {
    dvd_bnd: Arc<DvdBnd>,
}

impl DvdBndAssetSourcePlugin {
    pub fn new(
        data_archives: &[PathBuf],
        key_provider: impl ArchiveKeyProvider,
    ) -> io::Result<Self> {
        let dvd_bnd = Arc::new(DvdBnd::create(data_archives, &key_provider)?);

        Ok(Self { dvd_bnd })
    }
}

impl Plugin for DvdBndAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        let dvd_bnd = self.dvd_bnd.clone();

        app.register_asset_source(
            AssetSourceId::from("dvdbnd"),
            AssetSourceBuilder::new(move || Box::new(DvdBndAssetSource(dvd_bnd.clone()))),
        );
    }
}
