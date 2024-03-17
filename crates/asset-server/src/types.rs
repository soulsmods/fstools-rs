use bevy::prelude::*;

use crate::types::{
    bnd4::{Archive, ArchiveEntry, Bnd4Loader},
    flver::{FlverAsset, FlverLoader},
    msb::{MsbAsset, MsbAssetLoader, MsbPartAsset, MsbPointAsset},
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
            .register_asset_loader(MsbAssetLoader)
            .register_asset_loader(FlverLoader)
            .register_asset_loader(Bnd4Loader);
        app.init_asset::<MsbAsset>()
            .init_asset::<MsbPointAsset>()
            .init_asset::<MsbPartAsset>()
            .init_asset_loader::<MsbAssetLoader>();
    }
}
