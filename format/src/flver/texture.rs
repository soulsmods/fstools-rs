use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, F32, U32};

use crate::{flver::header::FlverHeaderPart, io_ext::zerocopy::Padding};

#[derive(FromBytes, FromZeroes)]
#[allow(unused)]
pub struct Texture<O: ByteOrder> {
    pub path_offset: U32<O>,
    pub type_offset: U32<O>,
    pub scale: [F32<O>; 2],
    unk10: u8,
    unk11: u8,
    padding0: Padding<2>,
    unk14: F32<O>,
    unk18: F32<O>,
    unk1c: F32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for Texture<O> {}
