use std::{collections::HashMap, error::Error};

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, Handle, LoadContext, RenderAssetUsages},
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    pbr::StandardMaterial,
    prelude::Mesh,
    reflect::{Reflect, TypePath},
};
use fstools_formats::flver::{
    face_set::FaceSetIndices, mesh::Mesh as FlverMesh, reader::VertexAttributeSemantic,
    vertex_buffer::accessor::VertexAttributeAccessor, Flver,
};
use tracing::info;

use crate::types::matbin::MatbinSettings;

#[derive(Asset, Debug, Reflect)]
pub struct FlverAsset {
    pub meshes: Vec<FlverMeshAsset>,
}

#[derive(Asset, Debug, Reflect)]
pub struct FlverMeshAsset {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

impl FlverAsset {
    pub fn meshes(&self) -> impl Iterator<Item = &FlverMeshAsset> {
        self.meshes.iter()
    }
}

#[derive(TypePath)]
pub struct FlverAssetLoader;

impl AssetLoader for FlverAssetLoader {
    type Asset = FlverAsset;

    type Settings = ();

    type Error = Box<dyn Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).await?;
        let flver = Flver::parse(&data)?;
        let mut meshes = Vec::with_capacity(flver.mesh_count());

        for (index, flver_mesh) in flver.meshes.iter().enumerate() {
            let material = flver.material(flver_mesh);
            let mtd_path = flver
                .material_mtd_path(material)
                .map(|path| path.to_path_buf())
                .ok_or(std::io::Error::other("corrupt data"))?;
            let offset = material.texture_index.get() as usize;
            let count = material.texture_count.get() as usize;
            let mut textures = HashMap::default();
            for tex in &flver.textures[offset..offset + count] {
                let name = flver.texture_type(tex).unwrap();
                let path = flver.texture_path(tex).unwrap();

                let path = path.to_path_buf().with_extension("dds").to_string();
                if path.is_empty() {
                    continue;
                }
                textures.insert(name.to_utf8(), path);
            }

            let material_handle = load_context
                .loader()
                .with_static_type()
                .with_settings(move |settings: &mut MatbinSettings| {
                    settings.textures = textures.clone();
                })
                .load(format!(
                    "vfs://materials/{}",
                    mtd_path
                        .with_extension("matbin")
                        .file_name()
                        .expect("has extension, name must be present")
                ));

            let mesh_handle = load_context.labeled_asset_scope(format!("mesh{index}"), |_ctx| {
                Ok::<_, Self::Error>(load_mesh(&flver, flver_mesh))
            })?;

            meshes.push(FlverMeshAsset {
                material: material_handle,
                mesh: mesh_handle,
            });
        }

        Ok(FlverAsset { meshes })
    }

    fn extensions(&self) -> &[&str] {
        &["flver"]
    }
}

fn load_mesh(flver: &Flver, flver_mesh: &FlverMesh) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
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
        use fstools_formats::flver::reader::VertexAttributeSemantic::{Normal, Position};

        let semantic = VertexAttributeSemantic::from(member.semantic_id.get());
        let Some(accessor) = flver.vertex_attribute_accessor(buffer, member) else {
            continue;
        };
        info!(?semantic, ?accessor);

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
            _ => {
                continue;
            }
        };

        mesh.insert_attribute(attribute, values);
    }

    let indices = match flver.face_set_indices(face_set) {
        Some(FaceSetIndices::U8(data)) => {
            Indices::U16(data.iter().map(|index| u16::from(*index)).collect())
        }
        Some(FaceSetIndices::U16(data)) => Indices::U16(data.iter().map(|val| val.get()).collect()),
        Some(FaceSetIndices::U32(data)) => Indices::U32(data.iter().map(|val| val.get()).collect()),
        _ => unimplemented!(),
    };

    mesh.insert_indices(indices);
    mesh.invert_winding().expect("wrong vertex topology");

    mesh
}
