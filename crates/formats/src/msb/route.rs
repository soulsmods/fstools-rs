use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, I32, U64};

use super::{MsbError, MsbParam, MsbVersion};
use crate::io_ext::read_wide_cstring;

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct ROUTE_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    index: I32<LE>,
}

impl<'a> MsbParam<'a, ROUTE_PARAM_ST<'a>, RouteType> for ROUTE_PARAM_ST<'a> {
    const NAME: &'static str = "ROUTE_PARAM_ST";

    fn read_entry(data: &'a [u8], _version: &'a MsbVersion) -> Result<Self, MsbError> {
        let inner = Inner::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[inner.name_offset.get() as usize..])?;

        Ok(ROUTE_PARAM_ST {
            name,
            unk8: inner.unk8,
            unkc: inner.unkc,
            unk10: inner.unk10,
            index: inner.index,
        })
    }

    fn of_type(
        routes: Result<impl Iterator<Item = Result<ROUTE_PARAM_ST<'a>, MsbError>>, MsbError>,
        _route_type: RouteType,
    ) -> Vec<ROUTE_PARAM_ST<'a>> {
        let mut routes_of_type: Vec<ROUTE_PARAM_ST<'a>> = vec![];

        if let Ok(routes) = routes {
            for part in routes.flatten() {
                routes_of_type.push(part);
            }
        }

        routes_of_type
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn type_index(&self) -> u32 {
        self.index.get() as u32
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
struct Inner {
    name_offset: U64<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    // Said to be some form of type?
    unk10: I32<LE>,
    index: I32<LE>,
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum RouteType {
    // TODO: Determine different route types
}
