use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
};

use self::{
    flver::{FLVERAsset, FLVERAssetLoader},
    tpf::{TPFAsset, TPFAssetLoader},
};

pub mod tpf;
pub mod flver;

#[derive(Default)]
pub struct FSFormatsAssetPlugin;

impl Plugin for FSFormatsAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TPFAsset>();
        app.init_asset_loader::<TPFAssetLoader>();
        app.init_asset::<FLVERAsset>();
        app.init_asset_loader::<FLVERAssetLoader>();
    }
}
