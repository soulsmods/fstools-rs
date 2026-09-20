use std::mem::size_of;

use byteorder::ByteOrder;
use zerocopy::{AsBytes, FromBytes, FromZeroes, F32, U32};

use crate::io_ext::zerocopy::Padding;

#[derive(AsBytes, FromZeroes, FromBytes)]
#[repr(C, packed)]
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

pub(crate) trait FlverHeaderPart: FromBytes + FromZeroes + Sized {
    fn from_indices_at<'a, I>(
        parts: &'a [Self],
        data: &'a [u8],
        indices_offset: usize,
        indices_count: usize,
    ) -> impl Iterator<Item = &'a Self>
    where
        I: Into<u32> + FromBytes + FromZeroes + Copy + 'static,
    {
        let data = &data[indices_offset..indices_offset + (indices_count * size_of::<I>())];
        let indices = I::slice_from(data).expect("buffer data was not aligned");

        indices.iter().map(|index| {
            let index = *index;
            &parts[index.into() as usize]
        })
    }
}
