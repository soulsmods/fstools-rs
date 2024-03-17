use std::sync::Arc;

use bevy::{
    asset::io::{AssetSource, AssetSourceId},
    prelude::*,
};
use fstools_vfs::Vfs;

use self::reader::VfsAssetRepository;

mod reader;

pub struct VfsAssetRepositoryPlugin {
    repository: VfsAssetRepository,
}

impl VfsAssetRepositoryPlugin {
    pub fn new(vfs: Vfs) -> Self {
        Self {
            repository: VfsAssetRepository(Arc::new(vfs)),
        }
    }
}

impl Plugin for VfsAssetRepositoryPlugin {
    fn build(&self, app: &mut App) {
        let repository = self.repository.clone();

        app.insert_resource(repository.clone());
        app.register_asset_source(
            AssetSourceId::from("vfs"),
            AssetSource::build().with_reader(move || Box::new(repository.clone())),
        );
    }
}
