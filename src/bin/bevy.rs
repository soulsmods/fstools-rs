use flver::{FLVERLayoutSemantic, FLVERLayoutType, FLVER};
use std::{
    f32::consts::TAU, fs, io::{self, SeekFrom}
};

use bevy::{
    gizmos, pbr::wireframe::{Wireframe, WireframeColor}, prelude::*, render::{mesh::{shape::{Cube, Plane}, Indices}, primitives::Sphere, render_resource::PrimitiveTopology}
};
use byteorder::{ReadBytesExt, LE};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

use std::io::{Read, Seek};

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO: accept from clap or some shit
    let mut file = fs::File::open("./samples/c3251.flver")
        .expect("Could not open input FLVER file");

    // Shit code
    let flver = FLVER::from_reader(&mut file).expect("Could not parse FLVER");
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
        let position_member = layout
            .members
            .iter()
            .filter(|i| {
                i.layout_semantic == FLVERLayoutSemantic::Position
                    && i.layout_type == FLVERLayoutType::Float3
            })
            .next();

        let normal_member = layout
            .members
            .iter()
            .filter(|i| {
                i.layout_semantic == FLVERLayoutSemantic::Normal
                    && i.layout_type == FLVERLayoutType::Byte4B
            })
            .next();

        if let Some(position_member) = position_member {
            file.seek(SeekFrom::Start(
                (flver.data_offset + vertex_buffer.buffer_offset) as u64
            ))
            .unwrap();

            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            for _ in 0..vertex_buffer.vertex_count {
                let mut buffer = vec![0x0u8; vertex_buffer.vertex_size as usize];
                file.read_exact(&mut buffer).unwrap();
                let mut cursor = io::Cursor::new(buffer);

                // Seek to XYZ vertex coords
                cursor.seek(SeekFrom::Start(position_member.struct_offset as u64))
                    .unwrap();

                vertices.push([
                    cursor.read_f32::<LE>().unwrap(),
                    cursor.read_f32::<LE>().unwrap(),
                    cursor.read_f32::<LE>().unwrap(),
                ]);

                if let Some(normal_member) = normal_member {
                    // Seek to normal member
                    cursor.seek(SeekFrom::Start(normal_member.struct_offset as u64))
                        .unwrap();

                    let x = (cursor.read_u8().unwrap() as i32 - 127) as f32 / 127.0;
                    let y = (cursor.read_u8().unwrap() as i32 - 127) as f32 / 127.0;
                    let z = (cursor.read_u8().unwrap() as i32 - 127) as f32 / 127.0;

                    normals.push([x, y, z]);
                } else {
                    normals.push([0.0, 0.0, 0.0]);
                }
            }

            let indices = match &face_set.indices {
                flver::FLVERFaceSetIndices::Byte0 => vec![],
                flver::FLVERFaceSetIndices::Byte1(i) => i.iter().map(|i| *i as u32).collect(),
                flver::FLVERFaceSetIndices::Byte2(i) => i.iter().map(|i| *i as u32).collect(),
                flver::FLVERFaceSetIndices::Byte4(i) => i.iter().map(|i| *i as u32).collect()
            };

            // Correct face order
            let indices = indices.chunks(3)
                .map(|i| [i[2], i[1], i[0]])
                .flatten()
                .collect::<Vec<u32>>();

            let mesh = Mesh::new(PrimitiveTopology::TriangleList)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_indices(Some(Indices::U32(indices)));

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(Color::GREEN.into()),
                    ..default()
                },
                Wireframe,
                WireframeColor { color: Color::GREEN },
            ));
        }
    }

    for dummy in flver.dummies.iter() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cube::new(0.1).into()),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_xyz(
                dummy.position.x,
                dummy.position.y,
                dummy.position.z,
            ),
            ..default()
        });
    }

    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::from_size(100.0).into()),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, 8.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
