use std::path::PathBuf;

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{AssetInspectorPlugin, WorldInspectorPlugin},
};
use clap::Parser;
use fstools_asset_server::{
    asset_source::vfs::{Vfs, VfsAssetSourcePlugin},
    types::{binder::Archive, flver::FlverAsset, Flver},
    DvdBndAssetSourcePlugin, FsFormatsPlugin,
};
use fstools_dvdbnd::recover_keys;
use fstools_game_id::GameId;
use fstools_game_installation::GameInstallation;

#[derive(Deref, DerefMut, Resource)]
pub struct ArchivesLoaded(Vec<Handle<Archive>>);

fn main() {
    let installation =
        GameInstallation::find(GameId::ELDEN_RING).expect("couldn't find installation");
    let keys = recover_keys(&installation.exe, &installation.bhds).expect("scanning keys");

    App::new()
        .add_plugins(
            DvdBndAssetSourcePlugin::new(&installation.bhds, keys).expect("assets_failure"),
        )
        .add_plugins(VfsAssetSourcePlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(FsFormatsPlugin)
        .add_plugins(InfiniteGridPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(AssetInspectorPlugin::<FlverAsset>::default())
        .add_plugins(AssetInspectorPlugin::<StandardMaterial>::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (setup_mounts, setup_scene).chain())
        .run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    erpath: Option<PathBuf>,
}

fn setup_scene(mut commands: Commands, assets: ResMut<AssetServer>) {
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn(InfiniteGridBundle::default());

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
    ));

    let flver: Handle<FlverAsset> = assets.load("vfs://parts/AM_M_1100.flver");
    let flver1: Handle<FlverAsset> = assets.load("vfs://parts/BD_M_2400.flver");
    commands.spawn((Transform::default(), Flver(flver)));
    commands.spawn((Transform::default(), Flver(flver1)));
}

fn setup_mounts(mut vfs: Vfs) {
    // TODO(gtierney): hm, no, I don't like this at all actually
    vfs.mount("parts", "dvdbnd://parts/bd_m_2400.partsbnd.dcx");
    vfs.mount("parts", "dvdbnd://parts/am_m_1100.partsbnd.dcx");
    vfs.mount("materials", "dvdbnd://material/allmaterial.matbinbnd.dcx");
    vfs.mount("texture", "vfs://parts/AM_M_1100.tpf");
    vfs.mount("texture", "vfs://parts/BD_M_2400.tpf");
    vfs.mount("texture", "dvdbnd://parts/common_body.tpf.dcx");
}
