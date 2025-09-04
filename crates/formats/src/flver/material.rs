use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::flver::header::FlverHeaderPart;

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Material<O: ByteOrder> {
    name_offset: U32<O>,
    mtd_name_offset: U32<O>,
    texture_count: U32<O>,
    texture_index: U32<O>,
    flags: U32<O>,
    gx_offset: U32<O>,
    unk18: U32<O>,
    unk1c: U32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for Material<O> {}
