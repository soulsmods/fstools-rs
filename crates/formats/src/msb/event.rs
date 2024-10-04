use std::borrow::Cow;

use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U16, U32, U64};

use super::{MsbError, MsbParam};
use crate::io_ext::{read_wide_cstring, zerocopy::Padding};

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct EVENT_PARAM_ST<'a> {
    name: &'a WStr<LE>,
    event_index: U32<LE>,
    event_type: I32<LE>,
    id: U32<LE>,
    event_data: EventData<'a>,
}

impl<'a> MsbParam<'a> for EVENT_PARAM_ST<'a> {
    const NAME: &'static str = "EVENT_PARAM_ST";

    fn read_entry(data: &'a [u8]) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[header.name_offset.get() as usize..])?;

        let event_data = EventData::from_type_and_slice(
            header.event_type.get(),
            &data[header.event_data_offset.get() as usize..],
        )?;

        Ok(EVENT_PARAM_ST {
            name,
            event_index: header.event_index,
            event_type: header.event_type,
            id: header.id,
            event_data,
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    event_index: U32<LE>,
    event_type: I32<LE>,
    id: U32<LE>,
    unk14: U32<LE>,
    general_data_offset: U64<LE>,
    event_data_offset: U64<LE>,
    unk3_offset: U64<LE>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum EventData<'a> {
    Other,
    Treasure(&'a EventDataTreasure),
    Generator(&'a EventDataGenerator),
    ObjAct(&'a EventDataObjAct),
    Navmesh(&'a EventDataNavmesh),
    PseudoMultiplayer(&'a EventDataPseudoMultiplayer),
    PlatoonInfo(&'a EventDataPlatoonInfo),
    PatrolInfo(&'a EventDataPatrolInfo),
    Mount(&'a EventDataMount),
    SignPool(&'a EventDataSignPool),
    RetryPoint(&'a EventDataRetryPoint),
}

impl<'a> EventData<'a> {
    pub fn from_type_and_slice(event_type: i32, data: &'a [u8]) -> Result<Self, MsbError> {
        Ok(match event_type {
            -1 => Self::Other,
            4 => Self::Treasure(
                EventDataTreasure::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            5 => Self::Generator(
                EventDataGenerator::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            7 => Self::ObjAct(
                EventDataObjAct::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            10 => Self::Navmesh(
                EventDataNavmesh::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            12 => Self::PseudoMultiplayer(
                EventDataPseudoMultiplayer::ref_from_prefix(data)
                    .ok_or(MsbError::UnalignedValue)?,
            ),
            15 => Self::PlatoonInfo(
                EventDataPlatoonInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            20 => Self::PatrolInfo(
                EventDataPatrolInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            21 => {
                Self::Mount(EventDataMount::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }
            23 => Self::SignPool(
                EventDataSignPool::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            24 => Self::RetryPoint(
                EventDataRetryPoint::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            _ => return Err(MsbError::UnknownEventDataType(event_type)),
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataTreasure {
    unk0: U32<LE>,
    unk4: U32<LE>,
    part_index: I32<LE>,
    unkc: U32<LE>,
    item_lot_param_1: I32<LE>,
    item_lot_param_2: I32<LE>,
    unk18: U32<LE>,
    unk1c: U32<LE>,
    unk20: U32<LE>,
    unk24: U32<LE>,
    unk28: U32<LE>,
    unk2c: U32<LE>,
    unk30: U32<LE>,
    unk34: U32<LE>,
    action_button_param: I32<LE>,
    pickup_animation: I32<LE>,
    in_chest: u8,
    start_disabled: u8,
    unk42: U16<LE>,
    unk44: U32<LE>,
    unk48: U32<LE>,
    unk4c: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataGenerator {
    max_num: u8,
    genenerator_type: u8,
    limit_num: I16<LE>,
    min_gen_num: I16<LE>,
    max_gen_num: I16<LE>,
    min_interval: F32<LE>,
    max_interval: F32<LE>,
    initial_spawn_count: u8,
    unk11: u8,
    unk12: u8,
    unk13: u8,
    unk14: F32<LE>,
    unk18: F32<LE>,
    unk1c: I32<LE>,
    unk20: I32<LE>,
    unk24: I32<LE>,
    unk28: I32<LE>,
    unk2c: I32<LE>,
    spawn_point_indices: [I32<LE>; 8],
    unk50: I32<LE>,
    unk54: I32<LE>,
    unk58: I32<LE>,
    unk5c: I32<LE>,
    spawn_part_indices: [I32<LE>; 32],
    unke0: I32<LE>,
    unke4: I32<LE>,
    unke8: I32<LE>,
    unkec: I32<LE>,
    unkf0: I32<LE>,
    unkf4: I32<LE>,
    unkf8: I32<LE>,
    unkfc: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataObjAct {
    entity_id: I32<LE>,
    part_index: I32<LE>,
    obj_act_param: I32<LE>,
    state_type: u8,
    paddingd: Padding<3>,
    event_flag_id: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataNavmesh {
    point_index: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataPseudoMultiplayer {
    host_entity_id: I32<LE>,
    event_flag_id: I32<LE>,
    activate_goods_id: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>, // Seems to be some event flag?
    unk14: I32<LE>,
    unk18: I32<LE>,
    ceremony_param: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataPlatoonInfo {
    platoon_id_script_active: I32<LE>,
    state: I32<LE>,
    un8: I32<LE>,
    unkc: I32<LE>,
    group_part_indices: [I32<LE>; 32],
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataPatrolInfo {
    unk0: u8,
    unk1: u8,
    unk2: u8,
    unk3: u8,
    unk4: I32<LE>,
    unk8: U32<LE>,
    unkc: U32<LE>,
    walk_point_indices: [I16<LE>; 64],
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataMount {
    rider_part_index: I32<LE>,
    mount_part_index: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataSignPool {
    sign_part_index: I32<LE>,
    sign_puddle_param: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct EventDataRetryPoint {
    retry_part_index: I32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    retry_region_index: I32<LE>,
}
