use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::types::{bnd4::Bnd4Plugin, flver::FlverPlugin, msb::MsbPlugin};

pub mod bnd4;
pub mod flver;
pub mod msb;

pub struct FsFormatsPlugins;

impl PluginGroup for FsFormatsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(Bnd4Plugin)
            .add(FlverPlugin)
            .add(MsbPlugin)
    }
}
