use std::{
    fmt::{Debug, Formatter},
    io::Read,
    ops::Deref,
};

use byteorder::{ByteOrder, LE};
use zerocopy::{AsBytes, FromBytes, FromZeroes, Ref, F32, U32};

use crate::{
    flver::{
        bone::Bone,
        dummy::Dummy,
        face_set::FaceSet,
        material::Material,
        mesh::Mesh,
        texture::Texture,
        vertex_buffer::{VertexBuffer, VertexBufferLayout},
    },
    io_ext::{zerocopy::Padding, ReadFormatsExt},
};

pub mod accessor;
mod bone;
mod dummy;
mod face_set;
mod material;
mod mesh;
pub mod reader;
mod texture;
mod vertex_buffer;

pub type Flver<'a> = FlverInner<'a, LE>;

#[allow(unused)]
pub struct FlverInner<'a, O: ByteOrder> {
    header: &'a FlverHeader<O>,
    data: &'a [u8],
    bones: &'a [Bone<O>],
    dummys: &'a [Dummy<O>],
    face_sets: &'a [FaceSet<O>],
    materials: &'a [Material<O>],
    meshes: &'a [Mesh<O>],
    textures: &'a [Texture<O>],
    vertex_buffers: &'a [VertexBuffer<O>],
    vertex_buffer_layouts: &'a [VertexBufferLayout<O>],
}

impl<'a, O: ByteOrder + 'static> Deref for FlverInner<'a, O> {
    type Target = FlverHeader<O>;

    fn deref(&self) -> &Self::Target {
        self.header
    }
}

impl<'a, O: ByteOrder + 'static> FlverInner<'a, O> {
    fn parse_no_verify(bytes: &'a [u8]) -> Option<Self> {
        let (header_ref, dummy_bytes) = Ref::<&'a [u8], FlverHeader<O>>::new_from_prefix(bytes)?;
        let header: &'a FlverHeader<O> = header_ref.into_ref();

        let (dummys, next) = Dummy::<O>::slice_from_prefix(dummy_bytes, header.dummy_count())?;

        let (materials, next) = Material::<O>::slice_from_prefix(next, header.material_count())?;

        let (bones, next) = Bone::<O>::slice_from_prefix(next, header.bone_count())?;

        let (meshes, next) = Mesh::<O>::slice_from_prefix(next, header.mesh_count())?;

        let (face_sets, next) = FaceSet::<O>::slice_from_prefix(next, header.face_set_count())?;

        let (vertex_buffers, next) =
            VertexBuffer::<O>::slice_from_prefix(next, header.vertex_buffer_count())?;

        let (vertex_buffer_layouts, next) =
            VertexBufferLayout::<O>::slice_from_prefix(next, header.vertex_buffer_layout_count())?;

        let (textures, _) = Texture::<O>::slice_from_prefix(next, header.texture_count())?;
        let data_offset = header.data_offset.get() as usize;
        let data_end = data_offset + header.data_length.get() as usize;
        let data = &bytes[data_offset..data_end];

        Some(Self {
            header,
            data,
            bones,
            dummys,
            face_sets,
            materials,
            meshes,
            textures,
            vertex_buffers,
            vertex_buffer_layouts,
        })
    }

    pub fn parse(data: &'a [u8]) -> Result<Self, std::io::Error> {
        let mut header = &data[..8];
        header.read_magic(b"FLVER\0")?;

        let mut endianness = vec![0x0u8; 2];
        header.read_exact(&mut endianness)?;

        let is_little_endian = endianness == [0x4c, 0x00];
        if !is_little_endian {
            return Err(std::io::Error::other(
                "only little endian FLVERs are supported",
            ));
        }

        Self::parse_no_verify(data).ok_or_else(|| std::io::Error::other("FLVER data is unaligned"))
    }
}

impl<'a, O: ByteOrder + 'static> Debug for FlverInner<'a, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flver")
            .field("version", &self.version.get())
            .field("data_offset", &self.data_offset.get())
            .field("data_length", &self.data_length.get())
            .field("dummy_count", &self.dummy_count.get())
            .field("material_count", &self.material_count.get())
            .field("mesh_count", &self.mesh_count.get())
            .field("vertex_buffer_count", &self.vertex_buffer_count.get())
            .field("bounding_box_min", &self.bounding_box_min)
            .field("bounding_box_max", &self.bounding_box_max)
            .field("face_count", &self.face_count.get())
            .field("total_face_count", &self.total_face_count.get())
            .field("vertex_index_size", &self.vertex_index_size)
            .field("unk_68", &self._unk68.get())
            .finish()
    }
}

#[derive(AsBytes, FromZeroes, FromBytes)]
#[repr(packed)]
#[allow(unused)]
pub struct FlverHeader<O: ByteOrder> {
    #[doc(hidden)]
    _padding0: Padding<8>,
    pub(crate) version: U32<O>,
    pub(crate) data_offset: U32<O>,
    pub(crate) data_length: U32<O>,
    pub(crate) dummy_count: U32<O>,
    pub(crate) material_count: U32<O>,
    pub(crate) bone_count: U32<O>,
    pub(crate) mesh_count: U32<O>,
    pub(crate) vertex_buffer_count: U32<O>,
    pub(crate) bounding_box_min: [F32<O>; 3],
    pub(crate) bounding_box_max: [F32<O>; 3],
    pub(crate) face_count: U32<O>,
    pub(crate) total_face_count: U32<O>,
    pub(crate) vertex_index_size: u8,
    pub(crate) unicode: u8,
    pub(crate) _unk4a: u8,
    pub(crate) _unk4b: u8,
    pub(crate) _unk4c: U32<O>,
    pub(crate) face_set_count: U32<O>,
    pub(crate) buffer_layout_count: U32<O>,
    pub(crate) texture_count: U32<O>,
    pub(crate) _unk5c: u8,
    pub(crate) _unk5d: u8,
    #[doc(hidden)]
    _padding1: Padding<10>,
    pub(crate) _unk68: U32<O>,

    #[doc(hidden)]
    _padding2: Padding<20>,
}

impl<O: ByteOrder + 'static> FlverHeader<O> {
    pub fn bone_count(&self) -> usize {
        self.bone_count.get() as usize
    }

    pub fn dummy_count(&self) -> usize {
        self.dummy_count.get() as usize
    }

    pub fn face_set_count(&self) -> usize {
        self.face_set_count.get() as usize
    }

    pub fn material_count(&self) -> usize {
        self.material_count.get() as usize
    }

    pub fn mesh_count(&self) -> usize {
        self.mesh_count.get() as usize
    }

    pub fn vertex_buffer_count(&self) -> usize {
        self.vertex_buffer_count.get() as usize
    }

    pub fn vertex_buffer_layout_count(&self) -> usize {
        self.buffer_layout_count.get() as usize
    }

    pub fn texture_count(&self) -> usize {
        self.texture_count.get() as usize
    }
}
