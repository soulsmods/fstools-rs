use std::error::Error;

use bevy::{
    app::{App, Plugin},
    asset::{
        io::Reader, Asset, AssetApp, AssetLoader, AsyncReadExt, BoxedFuture, Handle, LoadContext,
    },
    prelude::{Mesh, Reflect},
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
};
use fstools_formats::flver::{
    face_set::FaceSetIndices, mesh::Mesh as FlverMesh, reader::VertexAttributeSemantic,
    vertex_buffer::accessor::VertexAttributeAccessor, Flver,
};

pub struct FlverPlugin;

impl Plugin for FlverPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FlverAsset>()
            .register_type::<FlverAsset>()
            .register_asset_loader(FlverLoader);
    }
}

#[derive(Default)]
pub struct FlverLoader;

#[derive(Asset, Debug, Reflect)]
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
        let flver = Flver::parse(bytes)?;
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
