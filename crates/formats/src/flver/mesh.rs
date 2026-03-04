use byteorder::{ByteOrder, LE};
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::{flver::header::FlverHeaderPart, io_ext::zerocopy::Padding};

#[derive(Debug, FromZeroes, FromBytes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Mesh<O: ByteOrder = LE> {
    pub(crate) dynamic: u8,
    pub(crate) _padding1: Padding<3>,
    pub(crate) material_index: U32<O>,
    pub(crate) _padding2: Padding<8>,
    pub(crate) default_bone_index: U32<O>,
    pub(crate) bone_count: U32<O>,
    pub(crate) bounding_box_offset: U32<O>,
    pub(crate) bone_offset: U32<O>,
    pub(crate) face_set_count: U32<O>,
    pub(crate) face_set_offset: U32<O>,
    pub(crate) vertex_buffer_count: U32<O>,
    pub vertex_buffer_offset: U32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for Mesh<O> {}
