use bevy::prelude::*;

use self::{flver::FlverAssetLoader, msb::MsbAssetLoader};
use crate::types::{
    binder::{Archive, ArchiveEntry, Bnd4Loader},
    flver::FlverAsset,
    matbin::MatbinAssetLoader,
    msb::MsbAsset,
    tpf::TpfAssetLoader,
};

pub mod binder;
pub mod flver;
pub mod matbin;
pub mod msb;
pub mod tpf;

pub struct FsFormatsPlugin;

#[derive(Component)]
pub struct Flver(pub Handle<FlverAsset>);

impl Plugin for FsFormatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FlverAsset>()
            .register_type::<FlverAsset>()
            .register_type::<Handle<FlverAsset>>()
            .register_asset_reflect::<FlverAsset>()
            .init_asset::<Archive>()
            .init_asset::<ArchiveEntry>()
            .init_asset::<MsbAsset>()
            .register_asset_loader(TpfAssetLoader)
            .register_asset_loader(FlverAssetLoader)
            .register_asset_loader(MsbAssetLoader)
            .register_asset_loader(Bnd4Loader)
            .register_asset_loader(MatbinAssetLoader)
            .add_systems(PostUpdate, spawn_flver_meshes);
    }
}

#[derive(Component)]
pub struct SpawnedFlver;

fn spawn_flver_meshes(
    mut commands: Commands,
    unprocessed_entities: Query<(Entity, &Flver), Without<SpawnedFlver>>,
    my_assets: Res<Assets<FlverAsset>>,
) {
    for (parent_entity, asset_component) in &unprocessed_entities {
        if let Some(asset_data) = my_assets.get(&asset_component.0) {
            for field in asset_data.meshes() {
                let child = commands
                    .spawn((
                        Mesh3d(field.mesh.clone()),
                        MeshMaterial3d(field.material.clone()),
                    ))
                    .id();

                commands.entity(parent_entity).add_child(child);
            }

            commands.entity(parent_entity).insert(SpawnedFlver);
        }
    }
}
