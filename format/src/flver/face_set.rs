use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U16, U32};

use crate::io_ext::zerocopy::Padding;

#[derive(FromZeroes, FromBytes)]
#[repr(packed)]
#[allow(unused)]
pub struct FaceSet<O: ByteOrder> {
    flags: U32<O>,
    triangle_strip: u8,
    cull_back_faces: u8,
    unk06: U16<O>,
    index_count: U32<O>,
    index_offset: U32<O>,
    unk: U32<O>,
    padding0: Padding<4>,
    index_size: U32<O>,
    padding1: Padding<4>,
}
