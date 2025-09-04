use std::{
    fmt::{Debug, Display, Formatter},
    io,
    io::SeekFrom,
};

use byteorder::{ReadBytesExt, LE};

use crate::io_ext::ReadFormatsExt;

const _ALLOWED_VERSIONS: [u32; 2] = [
    0x2001A, // Elden Ring
    0x20021, // Nightreign
];

pub struct FLVERPartContext {
    pub data_offset: u32,
}

pub trait FLVERPartReader {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error>
    where
        Self: Sized;
}

impl Debug for FLVER {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flver")
            .field("version", &self.version)
            .field("data_offset", &self.data_offset)
            .field("data_length", &self.data_length)
            .field("dummy_count", &self.dummies.len())
            .field("material_count", &self.materials.len())
            .field("mesh_count", &self.meshes.len())
            .field("vertex_buffer_count", &self.vertex_buffers.len())
            .field("bounding_box_min", &self.bounding_box_min)
            .field("bounding_box_max", &self.bounding_box_max)
            .field("face_count", &self.face_count)
            .field("total_face_count", &self.total_face_count)
            .field("vertex_index_size", &self.vertex_index_size)
            .finish()
    }
}

pub struct FLVER {
    pub version: u32,
    pub data_offset: u32,
    pub data_length: u32,

    pub bounding_box_min: FLVERVector3,
    pub bounding_box_max: FLVERVector3,

    pub face_count: u32,
    pub total_face_count: u32,
    pub vertex_index_size: u8,

    pub dummies: Vec<FLVERDummy>,
    pub materials: Vec<FLVERMaterial>,
    pub bones: Vec<FLVERBone>,
    pub meshes: Vec<FLVERMesh>,
    pub face_sets: Vec<FLVERFaceSet>,
    pub vertex_buffers: Vec<VertexBuffer>,
    pub buffer_layouts: Vec<VertexBufferLayout>,
    pub textures: Vec<FLVERTexture>,
}

impl FLVER {
    pub fn from_reader(r: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        let mut magic = vec![0x0u8; 6];
        r.read_exact(&mut magic)?;

        // TODO: actually use this?
        let mut endianness = vec![0x0u8; 2];
        r.read_exact(&mut endianness)?;

        // assert!(endianness != [0x4C, 0x00], "Input is not little endian!");
        let version = r.read_u32::<LE>()?;

        // assert!(ALLOWED_VERSIONS.contains(&version), "FLVER version not supported");

        let data_offset = r.read_u32::<LE>()?;
        let data_length = r.read_u32::<LE>()?;
        let part_context = FLVERPartContext { data_offset };

        let dummy_count = r.read_u32::<LE>()?;
        let material_count = r.read_u32::<LE>()?;
        let bone_count = r.read_u32::<LE>()?;
        let mesh_count = r.read_u32::<LE>()?;
        let vertex_buffer_count = r.read_u32::<LE>()?;

        let bounding_box_min = FLVERVector3::from_reader(r, &part_context)?;
        let bounding_box_max = FLVERVector3::from_reader(r, &part_context)?;

        let face_count = r.read_u32::<LE>()?;
        let total_face_count = r.read_u32::<LE>()?;
        let vertex_index_size = r.read_u8()?;

        let _unicode = r.read_u8()?;
        let _unk4a = r.read_u8()?;
        let _unk4b = r.read_u8()?;
        let _unk4c = r.read_u32::<LE>()?;

        let face_set_count = r.read_u32::<LE>()?;
        let buffer_layout_count = r.read_u32::<LE>()?;
        let texture_count = r.read_u32::<LE>()?;

        let _unk5c = r.read_u8()?;
        let _unk5d = r.read_u8()?;
        r.read_u8()?;
        r.read_u8()?;
        r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;
        let _unk68 = r.read_u32::<LE>()?;
        // println!("unk68: {}", _unk68);
        r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;

        let dummies = read_vec::<FLVERDummy>(r, &part_context, dummy_count as usize)?;
        let materials = read_vec::<FLVERMaterial>(r, &part_context, material_count as usize)?;
        let bones = read_vec::<FLVERBone>(r, &part_context, bone_count as usize)?;
        let meshes = read_vec::<FLVERMesh>(r, &part_context, mesh_count as usize)?;
        let face_sets = read_vec::<FLVERFaceSet>(r, &part_context, face_set_count as usize)?;
        let vertex_buffers =
            read_vec::<VertexBuffer>(r, &part_context, vertex_buffer_count as usize)?;
        let buffer_layouts =
            read_vec::<VertexBufferLayout>(r, &part_context, buffer_layout_count as usize)?;
        let textures = read_vec::<FLVERTexture>(r, &part_context, texture_count as usize)?;

        Ok(Self {
            version,
            data_offset,
            data_length,

            bounding_box_min,
            bounding_box_max,

            face_count,
            total_face_count,
            vertex_index_size,

            dummies,
            materials,
            bones,
            meshes,
            face_sets,
            vertex_buffers,
            buffer_layouts,
            textures,
        })
    }
}

#[derive(Debug)]
pub struct FLVERVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FLVERPartReader for FLVERVector3 {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            x: r.read_f32::<LE>()?,
            y: r.read_f32::<LE>()?,
            z: r.read_f32::<LE>()?,
        })
    }
}

impl Display for FLVERVector3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[x: {0}, y: {1}, z: {2}]", self.x, self.y, self.z)
    }
}

#[derive(Debug)]
pub struct FLVERVector2 {
    pub x: f32,
    pub y: f32,
}

impl FLVERPartReader for FLVERVector2 {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            x: r.read_f32::<LE>()?,
            y: r.read_f32::<LE>()?,
        })
    }
}

#[derive(Debug)]
pub struct FLVERColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl FLVERPartReader for FLVERColor {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            r: r.read_u8()?,
            g: r.read_u8()?,
            b: r.read_u8()?,
            a: r.read_u8()?,
        })
    }
}

#[derive(Debug)]
pub struct FLVERDummy {
    pub position: FLVERVector3,
    pub color: FLVERColor,
    pub forward: FLVERVector3,
    pub reference_id: u16,
    pub parent_bone_index: u16,
    pub upward: FLVERVector3,
    pub attach_bone_index: u16,
    pub flag_1: bool,
    pub use_upward_vector: bool,
    pub unk30: u32,
    pub unk34: u32,

    // Could be padding?
    pub unk38: u32,
    pub unk3c: u32,
}

impl FLVERPartReader for FLVERDummy {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            position: FLVERVector3::from_reader(r, c)?,
            color: FLVERColor::from_reader(r, c)?,
            forward: FLVERVector3::from_reader(r, c)?,
            reference_id: r.read_u16::<LE>()?,
            parent_bone_index: r.read_u16::<LE>()?,
            upward: FLVERVector3::from_reader(r, c)?,
            attach_bone_index: r.read_u16::<LE>()?,
            flag_1: r.read_u8()? == 0x1,
            use_upward_vector: r.read_u8()? == 0x1,
            unk30: r.read_u32::<LE>()?,
            unk34: r.read_u32::<LE>()?,
            unk38: r.read_u32::<LE>()?,
            unk3c: r.read_u32::<LE>()?,
        })
    }
}

#[derive(Debug)]
pub struct FLVERMaterial {
    pub name: String,
    pub mtd: String,
    pub texture_count: u32,
    pub texture_index: u32,
    pub flags: u32,
    pub gx_offset: u32,
    pub unk18: u32,
    pub unk1c: u32,
}

impl FLVERPartReader for FLVERMaterial {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let name_offset = r.read_u32::<LE>()?;
        let mtd_offset = r.read_u32::<LE>()?;

        let current_pos = r.stream_position()?;
        r.seek(SeekFrom::Start(name_offset as u64))?;
        let name = r.read_utf16::<LE>()?;
        r.seek(SeekFrom::Start(mtd_offset as u64))?;
        let mtd = r.read_utf16::<LE>()?;
        r.seek(SeekFrom::Start(current_pos))?;

        let texture_count = r.read_u32::<LE>()?;
        let texture_index = r.read_u32::<LE>()?;
        let flags = r.read_u32::<LE>()?;
        let gx_offset = r.read_u32::<LE>()?;
        let unk18 = r.read_u32::<LE>()?;
        let unk1c = r.read_u32::<LE>()?;

        Ok(Self {
            name,
            mtd,
            texture_count,
            texture_index,
            flags,
            gx_offset,
            unk18,
            unk1c,
        })
    }
}

#[derive(Debug)]
pub struct FLVERBone {
    pub name: String,
    pub bounding_box_min: FLVERVector3,
    pub bounding_box_max: FLVERVector3,
    pub translation: FLVERVector3,
    pub rotation: FLVERVector3,
    pub scale: FLVERVector3,
    pub parent_index: u16,
    pub child_index: u16,
    pub next_sibling_index: u16,
    pub previous_sibling_index: u16,
    pub unk3c: u32,
}

impl FLVERPartReader for FLVERBone {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let translation = FLVERVector3::from_reader(r, c)?;
        let name_offset = r.read_u32::<LE>()?;

        let rotation = FLVERVector3::from_reader(r, c)?;
        let parent_index = r.read_u16::<LE>()?;
        let child_index = r.read_u16::<LE>()?;
        let scale = FLVERVector3::from_reader(r, c)?;
        let next_sibling_index = r.read_u16::<LE>()?;
        let previous_sibling_index = r.read_u16::<LE>()?;
        let bounding_box_min = FLVERVector3::from_reader(r, c)?;
        let unk3c = r.read_u32::<LE>()?;
        let bounding_box_max = FLVERVector3::from_reader(r, c)?;

        // Deal with FS garbage zeroes
        r.seek(SeekFrom::Current(0x34))?;

        let current_pos = r.stream_position()?;
        r.seek(SeekFrom::Start(name_offset as u64))?;
        let name = r.read_utf16::<LE>()?;
        r.seek(SeekFrom::Start(current_pos))?;

        Ok(Self {
            name,
            bounding_box_min,
            bounding_box_max,
            translation,
            rotation,
            scale,
            parent_index,
            child_index,
            next_sibling_index,
            previous_sibling_index,
            unk3c,
        })
    }
}

#[derive(Debug)]
pub struct FLVERMesh {
    pub dynamic: bool,
    pub material_index: u32,
    pub default_bone_index: u32,
    pub bounding_box_offset: u32,
    pub bone_indices: Vec<u32>,
    pub face_set_indices: Vec<u32>,
    pub vertex_buffer_indices: Vec<u32>,
}

impl FLVERPartReader for FLVERMesh {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let dynamic = r.read_u8()? == 0x1;
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);

        let material_index = r.read_u32::<LE>()?;
        assert!(r.read_u32::<LE>()? == 0x0);
        assert!(r.read_u32::<LE>()? == 0x0);
        let default_bone_index = r.read_u32::<LE>()?;
        let bone_count = r.read_u32::<LE>()?;
        let bounding_box_offset = r.read_u32::<LE>()?;
        let bone_offset = r.read_u32::<LE>()?;
        let face_set_count = r.read_u32::<LE>()?;
        let face_set_offset = r.read_u32::<LE>()?;
        let vertex_buffer_count = r.read_u32::<LE>()?;
        let vertex_buffer_offset = r.read_u32::<LE>()?;

        let current = r.stream_position()?;

        r.seek(SeekFrom::Start(bone_offset as u64))?;
        let bone_indices = read_vec::<u32>(r, c, bone_count as usize)?;

        r.seek(SeekFrom::Start(face_set_offset as u64))?;
        let face_set_indices = read_vec::<u32>(r, c, face_set_count as usize)?;

        r.seek(SeekFrom::Start(vertex_buffer_offset as u64))?;
        let vertex_buffer_indices = read_vec::<u32>(r, c, vertex_buffer_count as usize)?;

        r.seek(SeekFrom::Start(current))?;

        Ok(Self {
            dynamic,
            material_index,
            default_bone_index,
            bounding_box_offset,
            bone_indices,
            face_set_indices,
            vertex_buffer_indices,
        })
    }
}

#[derive(Debug)]
pub struct FLVERFaceSetFlags(u32);

impl From<u32> for FLVERFaceSetFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

pub const FACESET_FLAG_LOD1: u32 = 0x01000000;
pub const FACESET_FLAG_LOD2: u32 = 0x02000000;
pub const FACESET_FLAG_EDGECOMPRESSED: u32 = 0x40000000;
pub const FACESET_FLAG_MOTIONBLUR: u32 = 0x80000000;

impl FLVERFaceSetFlags {
    pub fn is_main(&self) -> bool {
        self.0 == 0x0
    }
}

#[derive(Debug)]
pub struct FLVERFaceSet {
    pub flags: FLVERFaceSetFlags,
    pub triangle_strip: bool,
    pub cull_back_faces: bool,
    pub unk06: u16,
    pub indices: FLVERFaceSetIndices,
}

#[derive(Debug)]
pub enum FLVERFaceSetIndices {
    Byte0,
    Byte1(Vec<u8>),
    Byte2(Vec<u16>),
    Byte4(Vec<u32>),
}

impl FLVERPartReader for FLVERFaceSet {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let flags = r.read_u32::<LE>()?.into();
        let triangle_strip = r.read_u8()? == 0x1;
        let cull_back_faces = r.read_u8()? == 0x1;
        let unk06 = r.read_u16::<LE>()?;
        let index_count = r.read_u32::<LE>()?;
        let index_offset = r.read_u32::<LE>()?;
        r.read_u32::<LE>()?;
        assert!(r.read_u32::<LE>()? == 0x0);
        let index_size = r.read_u32::<LE>()?;
        assert!(r.read_u32::<LE>()? == 0x0);

        let current = r.stream_position()?;
        r.seek(SeekFrom::Start(index_offset as u64 + c.data_offset as u64))?;
        let indices = match index_size {
            0 => FLVERFaceSetIndices::Byte0,
            8 => FLVERFaceSetIndices::Byte1(read_vec::<u8>(r, c, index_count as usize)?),
            16 => FLVERFaceSetIndices::Byte2(read_vec::<u16>(r, c, index_count as usize)?),
            32 => FLVERFaceSetIndices::Byte4(read_vec::<u32>(r, c, index_count as usize)?),
            _ => panic!("Unhandled index size {}", index_size),
        };
        r.seek(SeekFrom::Start(current))?;

        Ok(Self {
            flags,
            triangle_strip,
            cull_back_faces,
            unk06,
            indices,
        })
    }
}

#[derive(Debug)]

pub struct VertexBuffer {
    pub buffer_index: u32,
    pub layout_index: u32,
    pub vertex_size: u32,
    pub vertex_count: u32,
    pub buffer_length: u32,
    pub buffer_offset: u32,
}

impl FLVERPartReader for VertexBuffer {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let buffer_index = r.read_u32::<LE>()?;
        let layout_index = r.read_u32::<LE>()?;
        let vertex_size = r.read_u32::<LE>()?;
        let vertex_count = r.read_u32::<LE>()?;
        assert!(r.read_u32::<LE>()? == 0x0);
        assert!(r.read_u32::<LE>()? == 0x0);
        let buffer_length = r.read_u32::<LE>()?;
        let buffer_offset = r.read_u32::<LE>()?;

        Ok(Self {
            buffer_index,
            layout_index,
            vertex_size,
            vertex_count,
            buffer_length,
            buffer_offset,
        })
    }
}

#[derive(Debug)]
pub struct VertexBufferLayout {
    pub members: Vec<FLVERBufferLayoutMember>,
}

impl FLVERPartReader for VertexBufferLayout {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let member_count = r.read_u32::<LE>()?;
        assert!(r.read_u32::<LE>()? == 0x0);
        assert!(r.read_u32::<LE>()? == 0x0);
        let member_offset = r.read_u32::<LE>()?;

        let current = r.stream_position()?;

        r.seek(SeekFrom::Start(member_offset as u64))?;
        let members = read_vec::<FLVERBufferLayoutMember>(r, c, member_count as usize)?;

        r.seek(SeekFrom::Start(current))?;

        Ok(Self { members })
    }
}

impl VertexBufferLayout {
    pub fn member_by_type(
        &self,
        member_type: VertexAttributeSemantic,
    ) -> Option<&FLVERBufferLayoutMember> {
        self.members.iter().find(|m| m.semantic == member_type)
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
// TODO: these come from soulsformats and probably have documented
// names in dx12
pub enum VertexAttributeFormat {
    Float2 = 0x1,
    Float3 = 0x2,
    Float4 = 0x3,
    Byte4A = 0x10,
    Byte4B = 0x11,
    Short2ToFloat2 = 0x12,

    // int to float 127
    Byte4C = 0x13,
    UV = 0x15,

    // int to float
    UVPair = 0x16,
    ShortBoneIndices = 0x18,
    Short4ToFloat4A = 0x1A,
    Short4ToFloat4B = 0x2E,
    Byte4E = 0x2F,
    EdgeCompressed = 0xF0,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VertexAttributeSemantic {
    Position,
    BoneWeights,
    BoneIndices,
    Normal,
    UV,
    Tangent,
    Bitangent,
    VertexColor,
}

impl From<u32> for VertexAttributeSemantic {
    fn from(value: u32) -> Self {
        match value {
            0x0 => Self::Position,
            0x1 => Self::BoneWeights,
            0x2 => Self::BoneIndices,
            0x3 => Self::Normal,
            0x5 => Self::UV,
            0x6 => Self::Tangent,
            0x7 => Self::Bitangent,
            0xA => Self::VertexColor,
            _ => panic!("Unknown member type {}", value),
        }
    }
}

#[derive(Debug)]
pub struct FLVERBufferLayoutMember {
    pub unk0: u32,
    pub struct_offset: u32,
    pub format: u32,
    pub semantic: VertexAttributeSemantic,
    pub index: u32,
}

impl FLVERPartReader for FLVERBufferLayoutMember {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            unk0: r.read_u32::<LE>()?,
            struct_offset: r.read_u32::<LE>()?,
            format: r.read_u32::<LE>()?,
            semantic: r.read_u32::<LE>()?.into(),
            index: r.read_u32::<LE>()?,
        })
    }
}

#[derive(Debug)]
pub struct FLVERTexture {
    pub path: String,
    pub r#type: String,
    pub scale: FLVERVector2,
    pub unk10: u8,
    pub unk11: bool,
    pub unk14: f32,
    pub unk18: f32,
    pub unk1c: f32,
}

impl FLVERPartReader for FLVERTexture {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        let path_offset = r.read_u32::<LE>()?;
        let type_offset = r.read_u32::<LE>()?;

        let scale = FLVERVector2::from_reader(r, c)?;
        let unk10 = r.read_u8()?;
        let unk11 = r.read_u8()? == 0x1;
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);
        let unk14 = r.read_f32::<LE>()?;
        let unk18 = r.read_f32::<LE>()?;
        let unk1c = r.read_f32::<LE>()?;

        let current_pos = r.stream_position()?;
        r.seek(SeekFrom::Start(path_offset as u64))?;
        let path = r.read_utf16::<LE>()?;
        r.seek(SeekFrom::Start(type_offset as u64))?;
        let r#type = r.read_utf16::<LE>()?;
        r.seek(SeekFrom::Start(current_pos))?;

        Ok(Self {
            path,
            r#type,
            scale,
            unk10,
            unk11,
            unk14,
            unk18,
            unk1c,
        })
    }
}

impl FLVERPartReader for u8 {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        r.read_u8()
    }
}

impl FLVERPartReader for u16 {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        r.read_u16::<LE>()
    }
}

impl FLVERPartReader for u32 {
    fn from_reader(
        r: &mut (impl io::Read + io::Seek),
        _c: &FLVERPartContext,
    ) -> Result<Self, io::Error> {
        r.read_u32::<LE>()
    }
}

fn read_vec<T: FLVERPartReader>(
    r: &mut (impl io::Read + io::Seek),
    c: &FLVERPartContext,
    count: usize,
) -> Result<Vec<T>, io::Error> {
    let mut results = Vec::new();
    for _ in 0..count {
        results.push(T::from_reader(r, c)?);
    }

    Ok(results)
}
