use bevy::{
    app::{Plugin, PluginGroup, PluginGroupBuilder},
    asset::AssetApp,
};

use crate::flver::FlverPlugin;

pub mod tpf;

#[derive(Default)]
pub struct TpfPlugin;

pub struct FormatsPlugins;

impl PluginGroup for FormatsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(FlverPlugin)
            .add(TpfPlugin)
    }
}
