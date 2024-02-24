use std::{collections::HashMap, path::PathBuf};
use std::f32::consts::PI;
use std::io;
use std::io::Read;
use std::sync::RwLock;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{
    CompressedImageFormats, ImageAddressMode, ImageFormat, ImageSampler, ImageSamplerDescriptor,
    ImageType,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use vfs::VfsAssetRepositoryPlugin;
use formats::FSFormatsAssetPlugin;
use bevy::prelude::*;
use clap::Parser;
use steamlocate::SteamDir;
use souls_vfs::{FileKeyProvider, Vfs};

mod formats;
mod vfs;

const ER_APPID: u32 = 1245620;

fn locate_er_dir() -> PathBuf {
    let mut steamdir = SteamDir::locate().expect("steam installation not found");

    match steamdir.app(&ER_APPID) {
        Some(app) => app.path.join("Game"),
        None => panic!("couldn't find elden ring installation"),
    }
}

fn main() {
    let args = Args::parse();
    let er_path = args.erpath
        .unwrap_or_else(locate_er_dir);

    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];

    let mut vfs = Vfs::create(archives.clone(), &keys)
        .expect("unable to create vfs");

    vfs.mount("/parts/wp_a_0210.partsbnd.dcx")
        .expect("Could not mount bnd");

    vfs.mount("/chr/c3660_l.texbnd.dcx")
        .expect("Could not mount bnd");

    App::new()
        .add_plugins((VfsAssetRepositoryPlugin::new(vfs), DefaultPlugins))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FSFormatsAssetPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_dummies)
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
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
            mesh: meshes.add(
                Cuboid::new(2.0, 2.0, 2.0),
            ),
            material: material_handle,
            ..default()
        });
    }

    // From DCX'd TPF
    {
        let texture: Handle<Image> = asset_server.load("/asset/aet/aet230/aet230_557.tpf.dcx#AET230_557_a");
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(texture.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(
                Cuboid::new(2.0, 2.0, 2.0),
            ),
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            material: material_handle,
            ..default()
        },));
    }

    for dummy in flver.dummies.iter() {
        commands.spawn(Dummy {
            position: Vec3::new(dummy.position.x, dummy.position.y, dummy.position.z * -1.0),
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

    // Draw some floor
    let floor = flver.bounding_box_min;
    const FLOOR_DISTANCE: f32 = 0.2;
    commands.spawn(PbrBundle {
        transform: Transform {
            translation: Vec3::new(0.0, floor.y - FLOOR_DISTANCE, 0.0),
            ..default()
        },
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: materials.add(Color::ALICE_BLUE),
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

fn draw_dummies(mut gizmos: Gizmos, dummies: Query<&Dummy>) {
    for dummy in dummies.iter() {
        gizmos.sphere(dummy.position, Quat::IDENTITY, 0.02, Color::RED);
    }
}
