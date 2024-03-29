use std::error::Error;

use bevy::{
    asset::{Asset, Handle, LoadContext},
    prelude::Mesh,
    reflect::Reflect,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
};
use fstools_formats::flver::{
    face_set::FaceSetIndices, mesh::Mesh as FlverMesh, reader::VertexAttributeSemantic,
    vertex_buffer::accessor::VertexAttributeAccessor, Flver,
};

use crate::asset_source::fast_path::FastPathAssetLoader;

#[derive(Asset, Debug, Reflect)]
pub struct FlverAsset {
    meshes: Vec<Handle<Mesh>>,
}

impl FlverAsset {
    pub fn meshes(&self) -> impl Iterator<Item = &Handle<Mesh>> {
        self.meshes.iter()
    }
}

pub struct FlverAssetLoader;

impl FastPathAssetLoader for FlverAssetLoader {
    type Asset = FlverAsset;

    type Settings = ();

    type Error = Box<dyn Error + Send + Sync>;

    async fn load_from_bytes<'a>(
        reader: &'a [u8],
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let flver = Flver::parse(reader)?;
        let mut meshes = Vec::with_capacity(flver.mesh_count());

        for (index, flver_mesh) in flver.meshes.iter().enumerate() {
            let mesh_handle = load_context
                .labeled_asset_scope(format!("mesh{}", index), |_| load_mesh(&flver, flver_mesh));

            meshes.push(mesh_handle);
        }

        Ok(FlverAsset { meshes })
    }
}

fn load_mesh(flver: &Flver, flver_mesh: &FlverMesh) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let face_set = flver
        .mesh_face_sets(flver_mesh)
        .find(|set| set.is_lod0())
        .expect("couldn't find main face set");

    let buffer = flver
        .mesh_buffers(flver_mesh)
        .next()
        .expect("no vertex buffers for FLVER");

    let layout = &flver.vertex_buffer_layouts[buffer.layout_index.get() as usize];
    let layout_members = flver.vertex_attributes(layout);

    for member in layout_members {
        use fstools_formats::flver::reader::VertexAttributeSemantic::*;

        let semantic = VertexAttributeSemantic::from(member.semantic_id.get());
        let Some(accessor) = flver.vertex_attribute_accessor(buffer, member) else {
            continue;
        };

        let (attribute, values) = match (semantic, accessor) {
            (Position, VertexAttributeAccessor::Float3(it)) => (
                Mesh::ATTRIBUTE_POSITION,
                VertexAttributeValues::Float32x3(it.collect()),
            ),
            (Normal, VertexAttributeAccessor::Float3(it)) => (
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(it.collect()),
            ),
            (Normal, VertexAttributeAccessor::SNorm8x4(it)) => (
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(it.map(|f| [f[0], f[1], f[2]]).collect()),
            ),
            (UV, VertexAttributeAccessor::UV(it)) => (
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(it.collect()),
            ),
            _ => {
                continue;
            }
        };

        mesh.insert_attribute(attribute, values);
    }

    let indices = match flver.face_set_indices(face_set) {
        Some(FaceSetIndices::U8(data)) => {
            Indices::U16(data.iter().map(|index| *index as u16).collect())
        }
        Some(FaceSetIndices::U16(data)) => Indices::U16(data.iter().map(|val| val.get()).collect()),
        Some(FaceSetIndices::U32(data)) => Indices::U32(data.iter().map(|val| val.get()).collect()),
        _ => unimplemented!(),
    };

    mesh.insert_indices(indices);
    mesh
}
