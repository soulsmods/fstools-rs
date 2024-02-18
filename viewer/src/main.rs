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
use clap::Parser;
use steamlocate::SteamDir;

use format::flver::FLVER;
use format::tpf::TPF;
use souls_vfs::{FileKeyProvider, Vfs};
use util::{AssetRepository as AssetRepositoryImpl, FLVERMeshBuilder};


#[derive(Deref, DerefMut, Resource)]
pub struct AssetRepository(RwLock<AssetRepositoryImpl>);

#[derive(Component)]
struct Dummy {
    position: Vec3,
}

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
        .unwrap_or_else(|| locate_er_dir());

    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];
    let vfs = Vfs::create(archives, &keys).expect("unable to create vfs");
    let mut repository = AssetRepositoryImpl::new(vfs);

    // Load specified bnd4
    repository
        .mount_dcx_bnd4(&args.dcx)
        .expect("Could not load specified DCX");

    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugins(WorldInspectorPlugin::new())
        .insert_resource(AssetRepository(RwLock::new(repository)))
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
    mut textures: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    repository: Res<AssetRepository>,
) {
    let repository = repository.read().expect("unable to acquire read lock");
    // Attempt to find any flvers
    let flvers = repository.paths_by_extension("flver");
    let handle = flvers.first().expect("No FLVERs found in DCX");

    let flver = repository.file::<FLVER>(handle);
    let flver_bytes = io::Cursor::new(repository.file_bytes(handle));

    let mut mesh_builder = FLVERMeshBuilder::new(&flver, flver_bytes);

    let mut texture_handles = HashMap::<String, Handle<Image>>::new();
    let tpfs = repository.paths_by_extension("tpf");

    if let Some(handle) = tpfs.first() {
        let tpf = repository.file::<TPF>(handle);
        let mut tpf_bytes = io::Cursor::new(repository.file_bytes(handle));

        for texture in tpf.textures.iter() {
            let dds = texture.bytes(&mut tpf_bytes).unwrap();

            let image = Image::from_buffer(
                #[cfg(all(debug_assertions))]
                texture.name.clone(),
                &dds,
                ImageType::Format(ImageFormat::Dds),
                CompressedImageFormats::BC,
                false,
                ImageSampler::Descriptor(ImageSamplerDescriptor {
                    label: Some(texture.name.clone()),
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..Default::default()
                }),
                RenderAssetUsages::RENDER_WORLD,
            )
            .expect("Could not load image from DDS");

            texture_handles.insert(texture.name.clone(), textures.add(image));
        }
    }

    for mesh in mesh_builder.build().into_iter() {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uvs0)
        .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, mesh.tangents)
        .with_inserted_indices(Indices::U32(mesh.indices));

        let base_albedo_texture = texture_handles
            .keys()
            .find(|k| k.ends_with("_a"))
            .map(|k| &texture_handles[k]);

        let normal_map_texture = texture_handles
            .keys()
            .find(|k| k.ends_with("_n"))
            .map(|k| &texture_handles[k]);

        // let metallic_roughness_texture = texture_handles
        //     .keys()
        //     .find(|k| k.ends_with("_m"))
        //     .map(|k| &texture_handles[k]);

        commands.spawn((PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color_texture: base_albedo_texture.cloned(),

                // TODO: normal maps are weird rn, not sure what is up
                //normal_map_texture: normal_map_texture.cloned(),
                //flip_normal_map_y: true,
                ..default()
            }),
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
