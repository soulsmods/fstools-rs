use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, F32, U16};

use crate::io_ext::zerocopy::Padding;

pub trait FlverDummy {
    fn position(&self) -> (f32, f32, f32);
}

#[derive(FromZeroes, FromBytes)]
#[repr(packed)]
pub struct FlverDummyData<O: ByteOrder> {
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

impl<O: ByteOrder> FlverDummy for FlverDummyData<O> {
    fn position(&self) -> (f32, f32, f32) {
        (
            self.position[0].get(),
            self.position[1].get(),
            self.position[2].get(),
        )
    }
}
