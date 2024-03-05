use std::{f32::consts::PI, io, path::PathBuf};

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use clap::Parser;
use souls_vfs::{FileKeyProvider, Vfs};
use vfs::VfsAssetRepositoryPlugin;

use crate::{flver::asset::FlverAsset, formats::FormatsPlugins};

pub mod flver;
mod formats;
mod vfs;

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

    let mut vfs = Vfs::create(archives.clone(), &keys).expect("unable to create vfs");

    vfs.mount("/parts/wp_a_0210.partsbnd.dcx")
        .expect("Could not mount bnd");

    vfs.mount("/chr/c3660_l.texbnd.dcx")
        .expect("Could not mount bnd");

    App::new()
        .add_plugins((VfsAssetRepositoryPlugin::new(vfs), DefaultPlugins))
        .add_plugins(FormatsPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PanOrbitCameraPlugin)
        .init_resource::<AssetCollection>()
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_flvers)
        .run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    dcx: String,

    #[arg(long)]
    erpath: Option<PathBuf>,
}

#[derive(Debug)]
pub enum AssetLoadError {
    Io(io::Error),
    NotFound,
}

#[derive(Resource, Default)]
pub struct AssetCollection {
    assets: Vec<Handle<FlverAsset>>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<AssetCollection>,
    asset_server: Res<AssetServer>,
) {
    let flver: Handle<FlverAsset> = asset_server.load("wp_a_0210.flver"); 

    assets.assets.push(flver);
    // From mounted BND
    {
        let texture: Handle<Image> = asset_server.load("wp_a_0210.tpf#WP_A_0210_a");
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(texture.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 2.0, 2.0)),
            material: material_handle,
            ..default()
        });
    }

    // From DCX'd TPF
    {
        let texture: Handle<Image> =
            asset_server.load("/asset/aet/aet230/aet230_557.tpf.dcx#AET230_557_a");
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(texture.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 2.0, 2.0)),
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            material: material_handle,
            ..default()
        });
    }

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

pub fn spawn_flvers(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<FlverAsset>>,
    flvers: Res<Assets<FlverAsset>>,
) {
    for ev in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let flver = flvers.get(*id).expect("flver wasn't loaded");

            for mesh in flver.meshes() {
                commands.spawn(PbrBundle {
                    mesh: mesh.clone(),
                    transform: Transform::from_xyz(0.0, 5.0, 0.0),
                    ..PbrBundle::default()
                });
            }
        }
    }
}
