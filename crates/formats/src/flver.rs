use std::{
    fmt::{Debug, Formatter},
    io::Read,
    ops::Deref,
};

use byteorder::{ByteOrder, LE};
use header::FlverHeader;
use vertex_buffer::accessor::{
    VertexAttributeAccessor as Accessor, VertexAttributeAccessor, VertexAttributeIter as Iter,
};
use zerocopy::{FromBytes, Ref, U16, U32};

use crate::{
    flver::{
        bone::Bone,
        dummy::Dummy,
        face_set::{FaceSet, FaceSetIndices},
        header::FlverHeaderPart,
        material::Material,
        mesh::Mesh,
        texture::Texture,
        vertex_buffer::{VertexBuffer, VertexBufferAttribute, VertexBufferLayout},
    },
    io_ext::ReadFormatsExt,
};

pub mod bone;
pub mod dummy;
pub mod face_set;
mod header;
pub mod material;
pub mod mesh;
pub mod reader;
pub mod texture;
pub mod vertex_buffer;

pub type Flver<'a> = FlverInner<'a, LE>;

#[allow(unused)]
pub struct FlverInner<'a, O: ByteOrder> {
    header: &'a FlverHeader<O>,

    /// The entire underlying byte array this FLVER was created from.
    bytes: &'a [u8],

    /// The data region of this FLVER, containing vertex buffers and strings.
    data: &'a [u8],
    bones: &'a [Bone<O>],
    dummys: &'a [Dummy<O>],
    pub face_sets: &'a [FaceSet<O>],
    materials: &'a [Material<O>],
    pub meshes: &'a [Mesh<O>],
    textures: &'a [Texture<O>],
    pub vertex_buffers: &'a [VertexBuffer<O>],
    pub vertex_buffer_layouts: &'a [VertexBufferLayout<O>],
}

impl<'a, O: ByteOrder> FlverInner<'a, O> {}

impl<'a, O: ByteOrder + 'static> Deref for FlverInner<'a, O> {
    type Target = FlverHeader<O>;

    fn deref(&self) -> &Self::Target {
        self.header
    }
}

impl<'a, O: ByteOrder + 'static> FlverInner<'a, O> {
    pub fn face_set_indices(&self, face_set: &'a FaceSet<O>) -> Option<FaceSetIndices<'a, O>> {
        let index_size = face_set.index_size.get() as usize;
        let index_count = face_set.index_count.get() as usize;
        let index_offset = face_set.index_offset.get() as usize;
        let index_data = &self.data[index_offset..index_offset + (index_size / 8 * index_count)];

        Some(match face_set.index_size.get() {
            8 => FaceSetIndices::U8(index_data),
            16 => FaceSetIndices::U16(U16::slice_from(index_data)?),
            32 => FaceSetIndices::U32(U32::slice_from(index_data)?),
            _ => return None,
        })
    }

    pub fn mesh_buffers(&self, mesh: &'a Mesh<O>) -> impl Iterator<Item = &'a VertexBuffer<O>> {
        VertexBuffer::from_indices_at::<U32<O>>(
            self.vertex_buffers,
            self.bytes,
            mesh.vertex_buffer_offset.get() as usize,
            mesh.vertex_buffer_count.get() as usize,
        )
    }

    pub fn mesh_face_sets(&self, mesh: &'a Mesh<O>) -> impl Iterator<Item = &'a FaceSet<O>> {
        FaceSet::from_indices_at::<U32<O>>(
            self.face_sets,
            self.bytes,
            mesh.face_set_offset.get() as usize,
            mesh.face_set_count.get() as usize,
        )
    }

    pub fn vertex_attributes(
        &self,
        vertex_buffer_layout: &'a VertexBufferLayout<O>,
    ) -> &'a [VertexBufferAttribute<O>] {
        let attribute_count = vertex_buffer_layout.member_count.get() as usize;
        let attribute_offset = vertex_buffer_layout.member_offset.get() as usize;

        let (attrs, _) = VertexBufferAttribute::slice_from_prefix(
            &self.bytes[attribute_offset..],
            attribute_count,
        )
        .expect("unaligned_vertex_attributes");

        attrs
    }

    pub fn vertex_attribute_accessor(
        &self,
        buffer: &VertexBuffer<O>,
        attribute: &VertexBufferAttribute<O>,
    ) -> Option<VertexAttributeAccessor<'a>> {
        let buffer_offset = buffer.buffer_offset.get() as usize;
        let buffer_length = buffer.buffer_length.get() as usize;

        let data = &self.data[buffer_offset..buffer_offset + buffer_length];
        let vertex_size = buffer.vertex_size.get() as usize;
        let vertex_offset = attribute.struct_offset.get() as usize;

        use vertex_buffer::VertexFormat::*;

        attribute.format().map(|format| match format {
            Float32x3 => Accessor::Float3(Iter::new(data, vertex_size, vertex_offset)),
            Float32x2 => Accessor::Float2(Iter::new(data, vertex_size, vertex_offset)),
            Float32x4 => Accessor::Float4(Iter::new(data, vertex_size, vertex_offset)),
            Unorm8x4 => Accessor::UNorm8x4(Iter::new(data, vertex_size, vertex_offset)),
            Snorm8x4 => Accessor::SNorm8x4(Iter::new(data, vertex_size, vertex_offset)),
            Snorm16x4 => Accessor::SNorm16x4(Iter::new(data, vertex_size, vertex_offset)),
            Uint8x4 => Accessor::UNorm8x4(Iter::new(data, vertex_size, vertex_offset)),
            Sint16x4 => Accessor::SNorm16x4(Iter::new(data, vertex_size, vertex_offset)),
            Sscale16x2 => Accessor::SNorm16x2(Iter::new(data, vertex_size, vertex_offset)),
            Sscale16x4 => Accessor::SNorm16x4(Iter::new(data, vertex_size, vertex_offset)),
        })
    }

    fn parse_no_verify(bytes: &'a [u8]) -> Option<Self> {
        let (header_ref, dummy_bytes) = Ref::<_, FlverHeader<O>>::new_from_prefix(bytes)?;
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
            bytes,
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
