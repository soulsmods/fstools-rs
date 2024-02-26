use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, F32, U16, U32};

use crate::io_ext::zerocopy::Padding;

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
#[allow(unused)]
pub struct Bone<O: ByteOrder> {
    translation: [F32<O>; 3],
    name_offset: U32<O>,
    rotation: [F32<O>; 3],
    parent_index: U16<O>,
    child_index: U16<O>,
    scale: [F32<O>; 3],
    next_sibling_index: U16<O>,
    prev_sibling_index: U16<O>,
    bounding_box_min: [F32<O>; 3],
    unk3c: U32<O>,
    bounding_box_max: [F32<O>; 3],
    _padding0: Padding<0x34>,
}
