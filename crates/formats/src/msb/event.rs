pub mod elden_ring;
pub mod nightreign;

use std::fmt::{Debug, Formatter};

use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, I32, U32, U64};

use super::{MsbError, MsbParam, MsbVersion};
use crate::{
    io_ext::read_wide_cstring,
    msb::event::EventData::{EldenRing, Nightreign},
};

#[allow(unused, non_camel_case_types)]
pub struct EVENT_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    pub id: U32<LE>,
    pub general_data: &'a GeneralData,
    pub event_type: (I32<LE>, EventType),
    pub event_type_index: U32<LE>,
    pub event_data: EventData<'a>,
}

impl Debug for EVENT_PARAM_ST<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("name", &self.name.to_string())
            .field("id", &self.id.get())
            .field("event_type_index", &self.event_type_index.get())
            .field("", &self.general_data)
            .finish()
    }
}

impl<'a> EVENT_PARAM_ST<'a> {
    pub fn event_data(&self) -> &EventData<'a> {
        &self.event_data
    }
}

impl<'a> MsbParam<'a, EVENT_PARAM_ST<'a>, EventType> for EVENT_PARAM_ST<'a> {
    const NAME: &'static str = "EVENT_PARAM_ST";

    fn read_entry(data: &'a [u8], version: &'a MsbVersion) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[header.name_offset.get() as usize..])?;

        let general_data =
            GeneralData::ref_from_prefix(&data[header.general_data_offset.get() as usize..])
                .ok_or(MsbError::UnalignedValue)?;

        let event_type: EventType;
        let event_data: EventData;

        match version {
            MsbVersion::EldenRing => {
                event_type =
                    EventType::EldenRing(elden_ring::EventType::from(header.event_type.get()));
                event_data = EldenRing(elden_ring::EventData::from_type_and_slice(
                    header.event_type.get(),
                    &data[header.event_data_offset.get() as usize..],
                )?);
            }
            MsbVersion::Nightreign => {
                event_type =
                    EventType::Nightreign(nightreign::EventType::from(header.event_type.get()));
                event_data = Nightreign(nightreign::EventData::from_type_and_slice(
                    header.event_type.get(),
                    &data[header.event_data_offset.get() as usize..],
                )?);
            }
        };

        Ok(EVENT_PARAM_ST {
            name,
            id: header.id,
            general_data,
            event_type: (header.event_type, event_type),
            event_type_index: header.event_type_index,
            event_data,
        })
    }

    fn of_type(
        events: Result<impl Iterator<Item = Result<EVENT_PARAM_ST<'a>, MsbError>>, MsbError>,
        event_type: EventType,
    ) -> Vec<EVENT_PARAM_ST<'a>> {
        let mut group_events: Vec<EVENT_PARAM_ST<'a>> = vec![];

        if let Ok(events) = events {
            for event in events.flatten() {
                if event.event_type.1 == event_type {
                    group_events.push(event);
                }
            }
        }

        group_events
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn type_index(&self) -> u32 {
        self.event_type_index.get()
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    id: U32<LE>,
    event_type: I32<LE>,
    event_type_index: U32<LE>,
    unk14: U32<LE>,
    general_data_offset: U64<LE>,
    event_data_offset: U64<LE>,
    unk3_offset: U64<LE>,
}

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct GeneralData {
    pub part_index: I32<LE>,
    pub point_index: I32<LE>,
    pub entity_id: I32<LE>,
    pub unk0: I32<LE>,
}

impl Debug for GeneralData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeneralData")
            .field("part_index", &self.part_index.get())
            .field("point_index", &self.point_index.get())
            .field("entity_id", &self.entity_id.get())
            .field("unk0", &self.unk0.get())
            .finish()
    }
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum EventType {
    EldenRing(elden_ring::EventType),
    Nightreign(nightreign::EventType),
}

#[derive(Debug)]
#[allow(unused)]
pub enum EventData<'a> {
    EldenRing(elden_ring::EventData<'a>),
    Nightreign(nightreign::EventData<'a>),
}
