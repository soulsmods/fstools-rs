use byteorder::{ReadBytesExt, LE};
use format::flver::{
    FLVERBufferLayout, FLVERFaceSet, FLVERFaceSetIndices, FLVERMemberType, FLVERMesh, FLVERStorageType, FLVERVertexBuffer, FLVER
};
use std::io::{self, Read, Seek, SeekFrom};

pub struct FLVERMeshBuilder<'a> {
    flver: &'a FLVER,
    cursor: io::Cursor<&'a [u8]>,
}

#[derive(Debug, Default)]
pub struct FLVERMeshBuilderResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs0: Vec<[f32; 2]>,
    pub uvs1: Vec<[f32; 2]>,
    pub tangents: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

// TODO: cursed we should use a custom shader for this instead
impl<'a> FLVERMeshBuilder<'a> {
    pub fn new(flver: &'a FLVER, cursor: io::Cursor<&'a [u8]>) -> Self {
        Self { flver, cursor }
    }

    pub fn build(&mut self) -> Vec<FLVERMeshBuilderResult> {
        self.flver
            .meshes
            .iter()
            .map(|m| self.build_mesh(m))
            .collect()
    }

    fn build_mesh(&mut self, mesh: &'a FLVERMesh) -> FLVERMeshBuilderResult {
        let (layout, buffer) = Self::main_buffer_for_mesh(self.flver, mesh);
        let position_member = layout.member_by_type(FLVERMemberType::Position);
        // Bail on the mesh if there is no geometry to render
        if position_member.is_none() {
            return Default::default();
        }

        let position_member = position_member.unwrap();
        debug_assert!(position_member.storage_type == FLVERStorageType::Float3);

        let normal_member = layout.member_by_type(FLVERMemberType::Normal);
        let uv_member = dbg!(layout.member_by_type(FLVERMemberType::UV));
        let tangent_member = layout.member_by_type(FLVERMemberType::Tangent);

        let vec_capacity = buffer.vertex_count as usize;
        let mut positions = Vec::with_capacity(vec_capacity);
        let mut normals = Vec::with_capacity(vec_capacity);
        let mut uvs0 = Vec::with_capacity(vec_capacity);
        let mut uvs1 = Vec::with_capacity(vec_capacity);
        let mut tangents = Vec::with_capacity(vec_capacity);

        // Move to start of buffer in cursor
        self.cursor
            .seek(SeekFrom::Start(
                (self.flver.data_offset + buffer.buffer_offset) as u64,
            ))
            .unwrap();

        // Read all the vertex buffers
        for _ in 0..buffer.vertex_count {
            let mut buffer = vec![0x0u8; buffer.vertex_size as usize];
            self.cursor.read_exact(&mut buffer).unwrap();

            let mut entry_cursor = io::Cursor::new(buffer);

            let pos = PropertyAccessor::<{FLVERStorageType::Float3}>::read(
                &mut entry_cursor,
                position_member.struct_offset,
            ).expect("Could not read vertex positions");

            positions.push([pos[0], pos[1], pos[2] * -1.0]);

            if let Some(normal_member) = normal_member {
                debug_assert!(normal_member.storage_type == FLVERStorageType::Byte4B);
                let normal = PropertyAccessor::<{FLVERStorageType::Byte4B}>::read(
                    &mut entry_cursor,
                    normal_member.struct_offset,
                ).expect("Could not read normals");

                // Cursed mapping since FS stores these with an extra channel
                normals.push([normal[0], normal[1], normal[2]]);
            } else {
                normals.push([0.0, 0.0, 0.0]);
            }

            if let Some(uv_member) = uv_member {
                debug_assert!(uv_member.storage_type == FLVERStorageType::UVPair);

                let uvs = PropertyAccessor::<{FLVERStorageType::UVPair}>::read(
                    &mut entry_cursor,
                    uv_member.struct_offset,
                ).expect("Could not read UVs");

                uvs0.push([uvs[0], uvs[1]]);
                uvs1.push([uvs[2], uvs[3]]);
            } else {
                uvs0.push([0.0, 0.0]);
                uvs1.push([0.0, 0.0]);
            }

            if let Some(tangent_member) = tangent_member {
                debug_assert!(tangent_member.storage_type == FLVERStorageType::Byte4B);

                tangents.push(
                    PropertyAccessor::<{FLVERStorageType::Byte4B}>::read(
                        &mut entry_cursor,
                        tangent_member.struct_offset,
                    ).expect("Could not read tangents")
                );
            } else {
                tangents.push([0.0, 0.0, 0.0, 0.0]);
            }
        }

        let indices = match &self.main_face_set_for_mesh(mesh).indices {
            FLVERFaceSetIndices::Byte0 => vec![],
            FLVERFaceSetIndices::Byte1(i) => i.iter().map(|i| *i as u32).collect(),
            FLVERFaceSetIndices::Byte2(i) => i.iter().map(|i| *i as u32).collect(),
            FLVERFaceSetIndices::Byte4(i) => i.iter().map(|i| *i as u32).collect(),
        };

        FLVERMeshBuilderResult {
            positions,
            normals,
            uvs0,
            uvs1,
            tangents,
            indices,
        }
    }

    fn main_buffer_for_mesh(
        flver: &'a FLVER,
        mesh: &'a FLVERMesh,
    ) -> (&'a FLVERBufferLayout, &'a FLVERVertexBuffer) {
        let buffer = &flver.vertex_buffers[mesh.vertex_buffer_indices[0] as usize];
        let layout = &flver.buffer_layouts[buffer.layout_index as usize];

        (layout, buffer)
    }

    fn main_face_set_for_mesh(&self, mesh: &FLVERMesh) -> &FLVERFaceSet {
        let face_set = mesh
            .face_set_indices
            .iter()
            .map(|i| &self.flver.face_sets[*i as usize])
            .find(|i| i.flags.is_main())
            .expect("Could not find a main face set for the mesh");

        if face_set.triangle_strip == true {
            panic!("Triangle strip indices not supported");
        }

        face_set
    }
}

struct PropertyAccessor<const T: FLVERStorageType>;

impl PropertyAccessor<{FLVERStorageType::Float3}> {
    fn read(
        r: &mut (impl io::Read + io::Seek),
        offset: u32,
    ) -> Result<[f32; 3], io::Error> {
        r.seek(SeekFrom::Start(offset as u64))?;

        Ok([
            r.read_f32::<LE>()?,
            r.read_f32::<LE>()?,
            r.read_f32::<LE>()?,
        ])
    }
}

impl PropertyAccessor<{FLVERStorageType::Byte4B}> {
    fn read(
        r: &mut (impl io::Read + io::Seek),
        offset: u32,
    ) -> Result<[f32; 4], io::Error> {
        r.seek(SeekFrom::Start(offset as u64))?;

        Ok([
           (r.read_u8()? as u8 - 127) as f32 / 127.0,
           (r.read_u8()? as u8 - 127) as f32 / 127.0,
           (r.read_u8()? as u8 - 127) as f32 / 127.0,
           (r.read_u8()? as u8 - 127) as f32 / 127.0,
        ])
    }
}

impl PropertyAccessor<{FLVERStorageType::UVPair}> {
    fn read(
        r: &mut (impl io::Read + io::Seek),
        offset: u32,
    ) -> Result<[f32; 4], io::Error> {
        r.seek(SeekFrom::Start(offset as u64))?;

        Ok([
            r.read_u16::<LE>()? as f32 / 2048.0,
            r.read_u16::<LE>()? as f32 / 2048.0,
            r.read_u16::<LE>()? as f32 / 2048.0,
            r.read_u16::<LE>()? as f32 / 2048.0,
        ])
    }
}
