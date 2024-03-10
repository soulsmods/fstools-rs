use bevy::prelude::*;

use crate::flver::asset::{FlverAsset, FlverLoader};

pub mod asset;

pub struct FlverPlugin;

impl Plugin for FlverPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FlverAsset>()
            .init_asset_loader::<FlverLoader>();
    }
}
