use std::borrow::Cow;

use byteorder::LE;
use widestring::U16Str;
use zerocopy::{FromBytes, FromZeroes, U32, U64};

use crate::io_ext::read_widestring;

use super::{MsbError, MsbParam};

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct MODEL_PARAM_ST<'a> {
    pub name: Cow<'a, U16Str>,
    model_type: U32<LE>,
    id: U32<LE>,
    sib_path: Cow<'a, U16Str>,
    instance_count: U32<LE>,
}

impl<'a> MsbParam<'a> for MODEL_PARAM_ST<'a> {
    const NAME: &'static str = "MODEL_PARAM_ST";

    fn read_entry(data: &'a [u8]) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_widestring(&data[header.name_offset.get() as usize..])?;
        let sib_path = read_widestring(&data[header.sib_path_offset.get() as usize..])?;

        Ok(MODEL_PARAM_ST {
            name,
            sib_path,
            model_type: header.model_type,
            id: header.id,
            instance_count: header.instance_count,
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    model_type: U32<LE>,
    id: U32<LE>,
    sib_path_offset: U64<LE>,
    instance_count: U32<LE>,
}
