use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod tpf;

#[derive(Default)]
pub struct TpfPlugin;

pub struct FormatsPlugins;

impl PluginGroup for FormatsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(TpfPlugin)
    }
}
