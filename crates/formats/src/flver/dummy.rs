use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, F32, U16};

use crate::{flver::header::FlverHeaderPart, io_ext::zerocopy::Padding};

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Dummy<O: ByteOrder> {
    position: [F32<O>; 3],
    color: [u8; 4],
    forward: [F32<O>; 3],
    ref_id: U16<O>,
    parent_bone_index: U16<O>,
    up_vector: [F32<O>; 3],
    attached_bone_index: u16,
    flag_1: u8,
    use_up_vector: u8,
    _padding1: Padding<16>,
}

impl<O: ByteOrder> FlverHeaderPart for Dummy<O> {}
