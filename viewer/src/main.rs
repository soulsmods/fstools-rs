use bevy_egui::{EguiContext, EguiPlugin};
use bevy_inspector_egui::bevy_egui;
use bevy_inspector_egui::{
    bevy_inspector::hierarchy::SelectedEntities, DefaultInspectorConfigPlugin,
};
use bevy::render::texture::{CompressedImageFormats, ImageFormat, ImageSampler, ImageSamplerDescriptor, ImageType};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use format::bnd4::BND4;
use format::dcx::DCX;
use format::flver::{FLVERFaceSetIndices, FLVERMemberType, FLVERStorageType, FLVER};
use format::matbin::Matbin;
use format::tpf::TPF;
use std::collections;
use std::io::{self, SeekFrom};
use util::{GameArchive, GameArchiveError};

use clap::Parser;

#[derive(Component)]
struct Dummy {
    position: Vec3,
}

use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
    render::{
        mesh::{shape::Plane, Indices},
        render_resource::PrimitiveTopology,
    },
};
use bevy_inspector_egui::quick::{AssetInspectorPlugin, WorldInspectorPlugin};
use byteorder::{ReadBytesExt, LE};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AssetInspectorPlugin::<StandardMaterial>::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_dummies)
        .run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    archive: String,
    #[arg(long)]
    key: String,
    #[arg(long)]
    dcx: String,
}

#[derive(Debug)]
pub enum AssetLoadError {
    Io(io::Error),
    GameArchive(GameArchiveError),
    NotFound,
}

fn get_flver() -> Result<(Vec<u8>, Option<Vec<u8>>), AssetLoadError> {
    let args = Args::parse();

    let mut key_file = std::fs::File::open(args.key).expect("Could not open key fille");

    let mut key = Vec::new();
    key_file
        .read_to_end(&mut key)
        .expect("Key was not right size");

    let archive = GameArchive::new(&args.archive, &key).expect("Could not open game archive");

    let part_dcx = archive
        .file_bytes_by_path(&args.dcx)
        .map_err(AssetLoadError::GameArchive)?
        .ok_or(AssetLoadError::NotFound)?;

    let mut dcx_cursor = std::io::Cursor::new(part_dcx);
    let dcx = DCX::from_reader(&mut dcx_cursor).map_err(AssetLoadError::Io)?;

    let mut bnd4_cursor = std::io::Cursor::new(dcx.decompressed);
    let bnd4 = BND4::from_reader(&mut bnd4_cursor).map_err(AssetLoadError::Io)?;

    match bnd4
        .file_descriptors
        .iter()
        .find(|f| f.name.ends_with(".flver"))
    {
        Some(f) => {
            let tpf = bnd4
                .file_descriptors
                .iter()
                .find(|f| f.name.ends_with(".tpf"))
                .map(|f| {
                    f.bytes(&mut bnd4_cursor)
                        .map_err(AssetLoadError::Io)
                        .unwrap()
                });

            Ok((f.bytes(&mut bnd4_cursor).map_err(AssetLoadError::Io)?, tpf))
        }
        None => Err(AssetLoadError::NotFound),
    }
}

use std::io::{Read, Seek};

fn setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
) {
    let args = Args::parse();

    let mut key_file = std::fs::File::open(args.key).expect("Could not open key fille");

    let mut key = Vec::new();
    key_file
        .read_to_end(&mut key)
        .expect("Key was not right size");

    let archive = GameArchive::new(&args.archive, &key).expect("Could not open game archive");

    let (flver_bytes, tpf_bytes) = get_flver().unwrap();
    let mut flver_cursor = std::io::Cursor::new(flver_bytes);
    let flver = FLVER::from_reader(&mut flver_cursor).expect("Could not parse FLVER");

    let tpf = tpf_bytes.clone().map(|t| {
        let mut tpf_cursor = std::io::Cursor::new(t.as_slice());
        TPF::from_reader(&mut tpf_cursor).expect("Could not parse TPF")
    });

    let mut texture_handles = collections::HashMap::<String, Handle<Image>>::new();
    if let Some(tpf) = tpf {
        let mut tpf_cursor = std::io::Cursor::new(tpf_bytes.unwrap());

        for texture in tpf.textures {
            let dds = texture.bytes(&mut tpf_cursor).unwrap();

            let image = Image::from_buffer(
                &dds,
                ImageType::Format(ImageFormat::Dds),
                CompressedImageFormats::BC,
                false,
                ImageSampler::Descriptor(ImageSamplerDescriptor {
                    label: Some(texture.name.clone()),
                    ..Default::default()
                }),
            )
            .expect("Could not load image from DDS");

            let handle = textures.add(image);
            texture_handles.insert(texture.name, handle);
        }
    }

    let material_dcx_bytes = archive
        .file_bytes_by_path("/material/allmaterial.matbinbnd.dcx")
        .map_err(AssetLoadError::GameArchive)
        .unwrap()
        .ok_or(AssetLoadError::NotFound)
        .unwrap();

    let mut material_dcx_cursor = std::io::Cursor::new(material_dcx_bytes);
    let material = DCX::from_reader(&mut material_dcx_cursor)
        .map_err(AssetLoadError::Io)
        .unwrap();

    let mut material_bnd4_cursor = std::io::Cursor::new(material.decompressed);
    let material_bnd4 = BND4::from_reader(&mut material_bnd4_cursor)
        .map_err(AssetLoadError::Io)
        .unwrap();

    let mut matbins = collections::HashMap::<String, Matbin>::new();
    for material in flver.materials.iter() {
        println!("References mtd: {}", material.mtd);
        let mtd_descriptor = material_bnd4
            .file_descriptor_by_path_material(&material.mtd.as_str())
            .expect("Could not get MTD from materials DCX");

        let matbin_bytes = mtd_descriptor
            .bytes(&mut material_bnd4_cursor)
            .expect("Could not get bytes for matbin");
        let mut matbin_cursor = io::Cursor::new(matbin_bytes);

        let matbin = Matbin::from_reader(&mut matbin_cursor)
            .expect("Could not parse matbin");

        println!("Matbin: {:#?}", matbin);
    }

    // Shit code
    for mesh in flver.meshes.iter() {
        let face_set = mesh
            .face_set_indices
            .iter()
            .map(|i| &flver.face_sets[*i as usize])
            .find(|i| i.flags.is_main())
            .expect("Could not find a main face set for the mesh");

        if face_set.triangle_strip == true {
            panic!("Triangle strip indices not supported");
        }

        let vertex_buffer = &flver.vertex_buffers[mesh.vertex_buffer_indices[0] as usize];
        let layout = &flver.buffer_layouts[vertex_buffer.layout_index as usize];

        let position_member = layout.member_by_type(FLVERMemberType::Position);
        let normal_member = layout.member_by_type(FLVERMemberType::Normal);
        let uv_member = layout.member_by_type(FLVERMemberType::UV);
        let tangent_member = layout.member_by_type(FLVERMemberType::Tangent);

        if let Some(position_member) = position_member {
            assert!(position_member.storage_type == FLVERStorageType::Float3);

            flver_cursor
                .seek(SeekFrom::Start(
                    (flver.data_offset + vertex_buffer.buffer_offset) as u64,
                ))
                .unwrap();

            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut tangents = Vec::new();
            for _ in 0..vertex_buffer.vertex_count {
                let mut buffer = vec![0x0u8; vertex_buffer.vertex_size as usize];
                flver_cursor.read_exact(&mut buffer).unwrap();
                let mut cursor = io::Cursor::new(buffer);

                cursor
                    .seek(SeekFrom::Start(position_member.struct_offset as u64))
                    .unwrap();

                vertices.push([
                    cursor.read_f32::<LE>().unwrap(),
                    cursor.read_f32::<LE>().unwrap(),
                    cursor.read_f32::<LE>().unwrap() * -1.0,
                ]);

                if let Some(normal_member) = normal_member {
                    assert!(normal_member.storage_type == FLVERStorageType::Byte4B);

                    cursor
                        .seek(SeekFrom::Start(normal_member.struct_offset as u64))
                        .unwrap();

                    let x = (cursor.read_u8().unwrap() as i32 - 127) as f32 / 127.0;
                    let y = (cursor.read_u8().unwrap() as i32 - 127) as f32 / 127.0;
                    let z = (cursor.read_u8().unwrap() as i32 - 127) as f32 / 127.0;

                    normals.push([x, y, z * -1.0]);
                } else {
                    normals.push([0.0, 0.0, 0.0]);
                }

                if let Some(uv_member) = uv_member {
                    cursor
                        .seek(SeekFrom::Start(uv_member.struct_offset as u64))
                        .unwrap();

                    let x1 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;
                    let y1 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;
                    let x2 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;
                    let y2 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;

                    uvs.push([x1, y1]);
                } else {
                    uvs.push([0.0, 0.0]);
                }

                if let Some(tangent_member) = tangent_member {
                    cursor
                        .seek(SeekFrom::Start(tangent_member.struct_offset as u64))
                        .unwrap();

                    let x1 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;
                    let y1 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;
                    let x2 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;
                    let y2 = cursor.read_u16::<LE>().unwrap() as f32 / 2048.0;

                    tangents.push([x1, y1, x2, y2]);
                } else {
                    tangents.push([0.0, 0.0, 0.0, 0.0]);
                }
            }

            let indices = match &face_set.indices {
                FLVERFaceSetIndices::Byte0 => vec![],
                FLVERFaceSetIndices::Byte1(i) => i.iter().map(|i| *i as u32).collect(),
                FLVERFaceSetIndices::Byte2(i) => i.iter().map(|i| *i as u32).collect(),
                FLVERFaceSetIndices::Byte4(i) => i.iter().map(|i| *i as u32).collect(),
            };

            let mesh = Mesh::new(PrimitiveTopology::TriangleList)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, tangents)
                .with_indices(Some(Indices::U32(indices)))
                .with_generated_tangents()
                .expect("Could not generate tangents");

            let index = texture_handles.keys().find(|k| k.ends_with("_a"))
                .expect("Could not find albedo map in loaded textures");
            let base_color_texture = &texture_handles[index];

            let index = texture_handles.keys().find(|k| k.ends_with("_n"))
                .expect("Could not find albedo map in loaded textures");
            let normal_map_texture = &texture_handles[index];

            let index = texture_handles.keys().find(|k| k.ends_with("_m"))
                .expect("Could not find albedo map in loaded textures");
            let metallic_roughness_texture = &texture_handles[index];

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    //material: materials.add(Color::GREEN.into()),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(base_color_texture.clone()),
                        //normal_map_texture: Some(normal_map_texture.clone()),

                        // TODO: Needs inversion it seems
                        // metallic_roughness_texture: Some(metallic_roughness_texture.clone()),

                        ..default()
                    }),
                    ..default()
                },
            ));
        }
    }

    for dummy in flver.dummies.iter() {
        commands.spawn(Dummy {
            position: Vec3::new(
                dummy.position.x,
                dummy.position.y,
                dummy.position.z * -1.0,
            ),
        });
    }

    // let albedo_index = texture_handles.keys().find(|k| k.ends_with("_a"))
    //     .expect("Could not find albedo map in loaded textures");
    // let normal_index = texture_handles.keys().find(|k| k.ends_with("_n"))
    //     .expect("Could not find normal map in loaded textures");
    // let metallic_index = texture_handles.keys().find(|k| k.ends_with("_m"))
    //     .expect("Could not find metallic map in loaded textures");
    //
    // let base_color_texture = &texture_handles[albedo_index];
    // let normal_map_texture = &texture_handles[normal_index];
    // let metallic_roughness_texture = &texture_handles[metallic_index];
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Plane::from_size(4.0).into()),
    //     material: materials.add(StandardMaterial {
    //         base_color_texture: Some(base_color_texture.clone()),
    //         normal_map_texture: Some(normal_map_texture.clone()),
    //         metallic_roughness_texture: Some(metallic_roughness_texture.clone()),
    //         ..default()
    //     }),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     ..default()
    // });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, 8.0),
        ..default()
    });

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(50.0).into()),
    //     material: materials.add(Color::SILVER.into()),
    //     ..default()
    // });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

fn draw_dummies(
    mut gizmos: Gizmos,
    dummies: Query<&Dummy>,
) {
    for dummy in dummies.iter() {
        gizmos.sphere(
            dummy.position,
            Quat::IDENTITY,
            0.02,
            Color::RED,
        );
    }
}
