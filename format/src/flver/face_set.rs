use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U16, U32};

use crate::{flver::header::FlverHeaderPart, io_ext::zerocopy::Padding};

pub enum FaceSetIndices<'a, O> {
    None,
    U8(&'a [u8]),
    U16(&'a [U16<O>]),
    U32(&'a [U32<O>]),
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
pub struct FaceSet<O: ByteOrder> {
    flags: U32<O>,
    triangle_strip: u8,
    cull_back_faces: u8,
    unk06: U16<O>,
    pub(crate) index_count: U32<O>,
    pub(crate) index_offset: U32<O>,
    unk: U32<O>,
    padding0: Padding<4>,
    pub(crate) index_size: U32<O>,
    padding1: U32<O>,
}

impl<O: ByteOrder> FaceSet<O> {
    pub fn is_lod0(&self) -> bool {
        self.flags.get() == 0
    }
}

impl<O: ByteOrder> FlverHeaderPart for FaceSet<O> {}
