pub mod event;
pub mod model;
pub mod parts;
pub mod point;
pub mod route;

use byteorder::LE;
use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, Ref, U32, U64};

use self::{
    event::EVENT_PARAM_ST, model::MODEL_PARAM_ST, parts::PARTS_PARAM_ST, point::POINT_PARAM_ST,
    route::ROUTE_PARAM_ST,
};
use crate::{
    io_ext::{read_wide_cstring, ReadWidestringError},
    msb::{
        event::EventType, model::ModelType, parts::PartType, point::PointType, route::RouteType,
    },
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MsbVersion {
    EldenRing,
    Nightreign,
}

#[derive(Debug, Error)]
pub enum MsbError {
    #[error("Could not read string")]
    String(#[from] ReadWidestringError),

    #[error("Got unknown event data type {0}")]
    UnknownEventDataType(i32),

    #[error("Got unknown point data type {0}")]
    UnknownPointDataType(i32),

    #[error("Got unknown part data type {0}")]
    UnknownPartDataType(i32),

    #[error("Could not create reference to value")]
    UnalignedValue,

    #[error("Could not find requested param {0}")]
    ParamNotFound(&'static str),
}

#[allow(unused)]
pub struct Msb<'a> {
    version: &'a MsbVersion,

    bytes: &'a [u8],

    header: &'a Header,

    set_data: &'a [u8],
}

impl<'a> Msb<'a> {
    pub fn parse(bytes: &'a [u8], version: &'a MsbVersion) -> Result<Self, MsbError> {
        let (header, set_data) =
            Ref::<_, Header>::new_from_prefix(bytes).ok_or(MsbError::UnalignedValue)?;

        Ok(Self {
            version,
            bytes,
            header: header.into_ref(),
            set_data,
        })
    }

    pub fn models(
        &self,
    ) -> Result<impl Iterator<Item = Result<MODEL_PARAM_ST<'_>, MsbError>>, MsbError> {
        self.param_set::<_, ModelType>()
    }

    pub fn events(
        &self,
    ) -> Result<impl Iterator<Item = Result<EVENT_PARAM_ST<'_>, MsbError>>, MsbError> {
        self.param_set::<_, EventType>()
    }

    pub fn points(
        &self,
    ) -> Result<impl Iterator<Item = Result<POINT_PARAM_ST<'_>, MsbError>>, MsbError> {
        self.param_set::<_, PointType>()
    }

    pub fn routes(
        &self,
    ) -> Result<impl Iterator<Item = Result<ROUTE_PARAM_ST<'_>, MsbError>>, MsbError> {
        self.param_set::<_, RouteType>()
    }

    pub fn parts(
        &self,
    ) -> Result<impl Iterator<Item = Result<PARTS_PARAM_ST<'_>, MsbError>>, MsbError> {
        self.param_set::<_, PartType>()
    }

    /// Cycles over all the param sets until it's found one with a matching type identifier
    fn param_set<P, T>(&'a self) -> Result<impl Iterator<Item = Result<P, MsbError>> + 'a, MsbError>
    where
        P: MsbParam<'a, P, T> + Sized,
    {
        let mut current_slice = self.set_data;

        for _ in 0..6 {
            let (header, next) = Ref::<_, SetHeader>::new_from_prefix(current_slice)
                .ok_or(MsbError::UnalignedValue)?;

            let header = header.into_ref();

            let (offsets, next) =
                U64::<LE>::slice_from_prefix(next, header.count.get() as usize - 1)
                    .ok_or(MsbError::UnalignedValue)?;

            let name_offset = header.name_offset.get() as usize;

            if read_wide_cstring::<LE>(&self.bytes[name_offset..])?.to_string() == P::NAME {
                return Ok(offsets
                    .iter()
                    .map(|o| P::read_entry(&self.bytes[o.get() as usize..], self.version)));
            }

            let next_header_offset = U64::<LE>::ref_from_prefix(next)
                .ok_or(MsbError::UnalignedValue)?
                .get() as usize;

            current_slice = &self.bytes[next_header_offset..];
        }

        Err(MsbError::ParamNotFound(P::NAME))
    }
}

impl<'a> std::fmt::Debug for Msb<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Msb").field("header", self.header).finish()
    }
}

pub trait MsbParam<'a, P, T> {
    const NAME: &'static str;

    fn read_entry(data: &'a [u8], version: &'a MsbVersion) -> Result<Self, MsbError>
    where
        Self: Sized;

    /// Given a collection of params, return a vector containing only those of a certain type.
    ///
    /// Example: Given a collection of events and the event type `SignPool`,
    /// return a vector containing only `SignPool` events.
    fn of_type(
        params: Result<impl Iterator<Item = Result<P, MsbError>>, MsbError>,
        param_type: T,
    ) -> Vec<P>;

    fn name(&self) -> String;

    /// The index of this item relative to its type group
    fn type_index(&self) -> u32;
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    magic: [u8; 4],

    unk04: U32<LE>,

    /// Size of header from the magic to the next section
    header_size: U32<LE>,

    /// Big endian
    is_big_endian: u8,

    /// What the fuck?
    is_bit_big_endian: u8,

    /// Widestring strings being used?
    is_widestring: u8,

    /// Whether or not to use U64 offsets
    is_64_bit_offset: i8,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct SetHeader {
    /// Version of the param format.
    version: U32<LE>,

    /// Amount of param entries. Seems to be offset by +1. So an empty set
    /// will have a count of 1.
    count: U32<LE>,

    /// Offset to name of parameter.
    name_offset: U64<LE>,
}
