use std::path::PathBuf;

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_inspector_egui::quick::{AssetInspectorPlugin, WorldInspectorPlugin};
use clap::Parser;
use fstools_asset_server::{
    types::{bnd4::Archive, flver::FlverAsset},
    FsAssetSourcePlugin, FsFormatsPlugin,
};
use fstools_dvdbnd::FileKeyProvider;

use crate::{
    formats::FormatsPlugins,
    preload::{vfs_mount_system, ArchivesLoading},
};

mod formats;
mod preload;

fn main() {
    let args = Args::parse();
    let er_path = args.erpath.expect("no path to Elden Ring game provided");

    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];
    App::new()
        .add_plugins(FsAssetSourcePlugin::new(&archives, keys).expect("assets_failure"))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..Default::default()
        }))
        .add_plugins(FormatsPlugins)
        .add_plugins(FsFormatsPlugin)
        .add_plugins(AssetInspectorPlugin::<FlverAsset>::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CameraControllerPlugin)
        .add_plugins(WireframePlugin)
        .init_resource::<ArchivesLoading>()
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, vfs_mount_system)
        .run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    erpath: Option<PathBuf>,

    #[arg(long)]
    msb: String,
}

fn setup(
    mut commands: Commands,
    mut archives: ResMut<ArchivesLoading>,
    asset_server: Res<AssetServer>,
) {
    let archive: Handle<Archive> = asset_server.load("dvdbnd://parts/am_m_1100.partsbnd.dcx");
    archives.push(archive);
    let archive: Handle<Archive> = asset_server.load("dvdbnd://material/allmaterial.matbinbnd.dcx");
    archives.push(archive);

    let flver: Handle<FlverAsset> = asset_server.load("vfs://am_m_1100.flver");
    commands.spawn((SpatialBundle::default(), flver));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.5, 0.4, 0.0)),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        CameraController {
            walk_speed: 10.0,
            run_speed: 50.0,
            ..default()
        }
        .print_controls(),
    ));
}

#[derive(Component)]
pub struct FlverInstance;

#[allow(clippy::type_complexity)]
pub fn spawn_flvers(
    mut commands: Commands,
    mut flvers_to_spawn: Query<
        (Entity, &Handle<FlverAsset>),
        Or<(Without<FlverInstance>, Changed<Handle<FlverAsset>>)>,
    >,
    flvers: Res<Assets<FlverAsset>>,
) {
    for (entity, flver) in &mut flvers_to_spawn {
        let Some(flver_asset) = flvers.get(flver) else {
            continue;
        };

        commands
            .entity(entity)
            .despawn_descendants()
            .insert(FlverInstance)
            .with_children(|parent| {
                for mesh in flver_asset.meshes() {
                    parent.spawn(PbrBundle {
                        mesh: mesh.clone(),
                        ..PbrBundle::default()
                    });
                }
            });
    }
}
