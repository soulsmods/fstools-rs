use bevy::prelude::*;

use self::{flver::FlverAssetLoader, msb::MsbAssetLoader};
use crate::{
    asset_source::fast_path::FastPathAppExt,
    types::{
        bnd4::{Archive, ArchiveEntry, Bnd4Loader},
        flver::FlverAsset,
        msb::MsbAsset,
    },
};

pub mod bnd4;
pub mod flver;
pub mod msb;

pub struct FsFormatsPlugin;

impl Plugin for FsFormatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FlverAsset>()
            .register_type::<FlverAsset>()
            .register_type::<Handle<FlverAsset>>()
            .init_asset::<Archive>()
            .init_asset::<ArchiveEntry>()
            .init_asset::<MsbAsset>()
            .register_fast_path_loader(MsbAssetLoader)
            .register_fast_path_loader(FlverAssetLoader)
            .register_asset_loader(Bnd4Loader);
    }
}
