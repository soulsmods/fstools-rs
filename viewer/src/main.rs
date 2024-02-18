use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::sync;

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
use util::{AssetArchiveError, AssetRepository, FLVERMeshBuilder};

mod keys;

static ASSET_REPOSITORY: sync::OnceLock<sync::RwLock<AssetRepository>> = sync::OnceLock::new();

fn asset_repository_mut() -> sync::RwLockWriteGuard<'static, AssetRepository> {
    ASSET_REPOSITORY
        .get_or_init(default)
        .write()
        .expect("Couldn't obtain write lock")
}

fn asset_repository() -> sync::RwLockReadGuard<'static, AssetRepository> {
    ASSET_REPOSITORY
        .get_or_init(default)
        .read()
        .expect("Couldn't obtain read lock")
}

#[derive(Component)]
struct Dummy {
    position: Vec3,
}

const ER_APPID: u32 = 1245620;

fn main() {
    let args = Args::parse();
    ASSET_REPOSITORY.get_or_init(default);

    {
        let mut steamdir = SteamDir::locate().expect("steam installation not found");
        let er_path = match steamdir.app(&ER_APPID) {
            Some(app) => app.path.join("Game"),
            None => panic!("couldn't find elden ring installation")
        };

        let mut repository = asset_repository_mut();
        let keys = keys::eldenring_keys();
        let archives = [
            er_path.join("Data0"),
            er_path.join("Data1"),
            er_path.join("Data2"),
            er_path.join("Data3"),
            er_path.join("sd/sd"),
        ];

        for archive in archives {
            let archive_name = archive
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("invalid archive path");

            repository
                .mount_archive(archive.to_str().unwrap(), &keys[archive_name][..])
                .expect("Could not mount game archive");
        }

        // Load material bnd
        // repository.mount_dcx_bnd4("/material/allmaterial.matbinbnd.dcx")
        //     .expect("Could not mount material defs DCX");

        // Load specified bnd4
        repository
            .mount_dcx_bnd4(&args.dcx)
            .expect("Could not load specified DCX");
    }

    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugins(WorldInspectorPlugin::new())
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
}

#[derive(Debug)]
pub enum AssetLoadError {
    Io(io::Error),
    AssetArchive(AssetArchiveError),
    NotFound,
}

fn setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let repository = asset_repository();


    // Attempt to find any flvers
    let flvers = repository.paths_by_extension("flver");
    let handle = flvers.iter().next().expect("No FLVERs found in DCX");

    let flver = repository.file::<FLVER>(handle);
    let flver_bytes = io::Cursor::new(repository.file_bytes(handle));

    let mut mesh_builder = FLVERMeshBuilder::new(&flver, flver_bytes);

    let mut texture_handles = HashMap::<String, Handle<Image>>::new();
    let tpfs = repository.paths_by_extension("tpf");

    if let Some(handle) = tpfs.iter().next() {
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

    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            intensity: 150.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(2.0, 2.0, 2.0)
            .looking_at(Vec3::ZERO, Vec3::Y),

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
