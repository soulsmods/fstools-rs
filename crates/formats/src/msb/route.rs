use std::borrow::Cow;

use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, I32, U64};

use super::{MsbError, MsbParam};
use crate::io_ext::read_wide_cstring;

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct ROUTE_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    id: I32<LE>,
}

impl<'a> MsbParam<'a> for ROUTE_PARAM_ST<'a> {
    const NAME: &'static str = "ROUTE_PARAM_ST";

    fn read_entry(data: &'a [u8]) -> Result<Self, MsbError> {
        let inner = Inner::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[inner.name_offset.get() as usize..])?;

        Ok(ROUTE_PARAM_ST {
            name,
            unk8: inner.unk8,
            unkc: inner.unkc,
            unk10: inner.unk10,
            id: inner.id,
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
struct Inner {
    name_offset: U64<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    // Said to be some form of type?
    unk10: I32<LE>,
    id: I32<LE>,
}
