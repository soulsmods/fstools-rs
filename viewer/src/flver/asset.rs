use std::{error::Error, io::Cursor};

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, BoxedFuture, Handle, LoadContext},
    log::warn,
    prelude::{Mesh, TypePath},
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
};
use format::flver::{
    accessor::VertexAttributeAccessor, FLVERFaceSetIndices, FLVERMesh, VertexAttributeSemantic,
    FLVER,
};

#[derive(Default)]
pub struct FlverLoader;

#[derive(Asset, Debug, TypePath)]
pub struct FlverAsset {
    meshes: Vec<Handle<Mesh>>,
}

impl FlverAsset {
    pub fn meshes(&self) -> impl Iterator<Item = &Handle<Mesh>> {
        self.meshes.iter()
    }
}

impl AssetLoader for FlverLoader {
    type Asset = FlverAsset;
    type Settings = ();
    type Error = Box<dyn Error + Send + Sync>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<FlverAsset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            FlverLoader::load_flver(&bytes, load_context).await
        })
    }

    fn extensions(&self) -> &[&str] {
        &["flver"]
    }
}

impl FlverLoader {
    async fn load_flver<'a, 'data, 'ctx>(
        bytes: &'data [u8],
        load_context: &'a mut LoadContext<'ctx>,
    ) -> Result<FlverAsset, Box<dyn Error + Send + Sync>> {
        let mut reader = Cursor::new(bytes);
        let flver = FLVER::from_reader(&mut reader)?;
        let data = &bytes[flver.data_offset as usize..];
        let mut meshes = Vec::with_capacity(flver.meshes.len());

        for (index, flver_mesh) in flver.meshes.iter().enumerate() {
            let mesh_handle = load_context.labeled_asset_scope(format!("mesh{}", index), |_| {
                load_mesh(&flver, flver_mesh, data)
            });

            meshes.push(mesh_handle);
        }

        Ok(FlverAsset { meshes })
    }
}

fn load_mesh(flver: &FLVER, flver_mesh: &FLVERMesh, data: &[u8]) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let face_set = flver_mesh
        .face_set_indices
        .iter()
        .map(|i| &flver.face_sets[*i as usize])
        .find(|i| i.flags.is_main())
        .expect("Could not find a main face set for the mesh");

    let buffer = &flver.vertex_buffers[flver_mesh.vertex_buffer_indices[0] as usize];
    let layout = &flver.buffer_layouts[buffer.buffer_index as usize];

    for member in &layout.members {
        let accessor = buffer.accessor(member, data);

        use VertexAttributeSemantic::*;
        let (attribute, values) = match (member.semantic, accessor) {
            (Position, VertexAttributeAccessor::Float3(it)) => (
                Mesh::ATTRIBUTE_POSITION,
                VertexAttributeValues::Float32x3(it.collect()),
            ),
            (Normal, VertexAttributeAccessor::Float3(it)) => (
                Mesh::ATTRIBUTE_NORMAL,
                VertexAttributeValues::Float32x3(it.collect()),
            ),
            (UV, VertexAttributeAccessor::UV(it)) => (
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(it.collect()),
            ),
            _ => {
                warn!(
                    "Vertex Attribute {:#?} and format {:#?} is currently unsupported",
                    member.semantic, member.format
                );

                continue;
            }
        };

        mesh.insert_attribute(attribute, values);
    }

    let indices = match &face_set.indices {
        FLVERFaceSetIndices::Byte0 => unimplemented!(),
        FLVERFaceSetIndices::Byte1(data) => {
            Indices::U16(data.iter().map(|index| *index as u16).collect())
        }
        FLVERFaceSetIndices::Byte2(data) => Indices::U16(data.clone()),
        FLVERFaceSetIndices::Byte4(data) => Indices::U32(data.clone()),
    };

    mesh.insert_indices(indices);
    mesh
}
