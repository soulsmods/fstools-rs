pub mod elden_ring;
pub mod nightreign;

use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, F32, I32, U32, U64};

use super::{MsbError, MsbParam, MsbVersion};
use crate::{
    io_ext::read_wide_cstring,
    msb::point::PointData::{EldenRing, Nightreign},
};

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct POINT_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    pub shape_type: U32<LE>,
    pub position: [F32<LE>; 3],
    pub rotation: [F32<LE>; 3],
    pub point_type: (I32<LE>, PointType),
    pub point_type_index: U32<LE>,
    pub point: PointData<'a>,
}

impl<'a> MsbParam<'a, POINT_PARAM_ST<'a>, PointType> for POINT_PARAM_ST<'a> {
    const NAME: &'static str = "POINT_PARAM_ST";

    fn read_entry(data: &'a [u8], version: &'a MsbVersion) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[header.name_offset.get() as usize..])?;

        let point_type: PointType;
        let point: PointData;

        match version {
            MsbVersion::EldenRing => {
                point_type =
                    PointType::EldenRing(elden_ring::PointType::from(header.point_type.get()));
                point = EldenRing(elden_ring::PointData::from_type_and_slice(
                    header.point_type.get(),
                    &data[header.point_data_offset.get() as usize..],
                )?);
            }
            MsbVersion::Nightreign => {
                point_type =
                    PointType::Nightreign(nightreign::PointType::from(header.point_type.get()));
                point = Nightreign(nightreign::PointData::from_type_and_slice(
                    header.point_type.get(),
                    &data[header.point_data_offset.get() as usize..],
                )?);
            }
        };

        Ok(POINT_PARAM_ST {
            name,
            shape_type: header.shape_type,
            position: header.position,
            rotation: header.rotation,
            point_type: (header.point_type, point_type),
            point_type_index: header.point_type_index,
            point,
        })
    }

    fn of_type(
        points: Result<impl Iterator<Item = Result<POINT_PARAM_ST<'a>, MsbError>>, MsbError>,
        point_type: PointType,
    ) -> Vec<POINT_PARAM_ST<'a>> {
        let mut group_points: Vec<POINT_PARAM_ST<'a>> = vec![];

        if let Ok(points) = points {
            for point in points.flatten() {
                if point.point_type.1 == point_type {
                    group_points.push(point);
                }
            }
        }

        group_points
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn type_index(&self) -> u32 {
        self.point_type_index.get()
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    point_type: I32<LE>,
    point_type_index: U32<LE>,
    shape_type: U32<LE>,
    position: [F32<LE>; 3],
    rotation: [F32<LE>; 3],
    unk2c: U32<LE>,
    shorts_a_offset: U64<LE>,
    shorts_b_offset: U64<LE>,
    unk40: U32<LE>,
    map_studio_layer: U32<LE>,
    shape_data_offset: U64<LE>,
    entity_data_offset: U64<LE>,
    point_data_offset: U64<LE>,
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum PointType {
    EldenRing(elden_ring::PointType),
    Nightreign(nightreign::PointType),
}

#[derive(Debug)]
#[allow(unused)]
pub enum PointData<'a> {
    EldenRing(elden_ring::PointData<'a>),
    Nightreign(nightreign::PointData<'a>),
}
