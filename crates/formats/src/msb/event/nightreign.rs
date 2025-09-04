use std::fmt::{Debug, Formatter};

use byteorder::LE;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U16, U32};

use super::MsbError;
use crate::io_ext::zerocopy::Padding;

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum EventType {
    Other = -1,
    Treasure = 4,
    Generator = 5,
    ObjAct = 7,
    PlatoonInfo = 15,
    PatrolInfo = 20,
    Mount = 21,
    BirdTravelInfo = 25,
    TalkInfo = 26,
    GroupBattleInfo = 27,
    Unknown,
}

impl EventType {
    pub fn variants() -> Vec<(EventType, &'static str)> {
        vec![
            (EventType::Other, "Other"),
            (EventType::Treasure, "Treasure"),
            (EventType::Generator, "Generator"),
            (EventType::ObjAct, "ObjAct"),
            (EventType::PlatoonInfo, "PlatoonInfo"),
            (EventType::PatrolInfo, "PatrolInfo"),
            (EventType::Mount, "Mount"),
            (EventType::BirdTravelInfo, "BirdTravelInfo"),
            (EventType::TalkInfo, "TalkInfo"),
            (EventType::GroupBattleInfo, "GroupBattleInfo"),
        ]
    }
}

impl From<i32> for EventType {
    fn from(v: i32) -> Self {
        match v {
            -1 => EventType::Other,
            4 => EventType::Treasure,
            5 => EventType::Generator,
            7 => EventType::ObjAct,
            15 => EventType::PlatoonInfo,
            20 => EventType::PatrolInfo,
            21 => EventType::Mount,
            25 => EventType::BirdTravelInfo,
            26 => EventType::TalkInfo,
            27 => EventType::GroupBattleInfo,
            _ => EventType::Unknown,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub enum EventData<'a> {
    Other,
    Treasure(&'a EventDataTreasure),
    Generator(&'a EventDataGenerator),
    ObjAct(&'a EventDataObjAct),
    PlatoonInfo(&'a EventDataPlatoonInfo),
    PatrolInfo(&'a EventDataPatrolInfo),
    Mount(&'a EventDataMount),
    BirdTravelInfo(&'a EventDataBirdTravelInfo),
    TalkInfo(&'a EventDataTalkInfo),
    GroupBattleInfo(&'a EventDataGroupBattleInfo),
}

impl<'a> EventData<'a> {
    pub fn from_type_and_slice(event_type_id: i32, data: &'a [u8]) -> Result<Self, MsbError> {
        let event_type = EventType::from(event_type_id);
        Ok(match event_type {
            EventType::Other => Self::Other,
            EventType::Treasure => Self::Treasure(
                EventDataTreasure::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::Generator => Self::Generator(
                EventDataGenerator::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::ObjAct => Self::ObjAct(
                EventDataObjAct::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::PlatoonInfo => Self::PlatoonInfo(
                EventDataPlatoonInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::PatrolInfo => Self::PatrolInfo(
                EventDataPatrolInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::Mount => {
                Self::Mount(EventDataMount::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }
            EventType::BirdTravelInfo => Self::BirdTravelInfo(
                EventDataBirdTravelInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::TalkInfo => Self::TalkInfo(
                EventDataTalkInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            EventType::GroupBattleInfo => Self::GroupBattleInfo(
                EventDataGroupBattleInfo::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            _ => return Err(MsbError::UnknownEventDataType(event_type_id)),
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct EventDataObjAct {
    entity_id: I32<LE>,
    part_index: I32<LE>,
    obj_act_param: I32<LE>,
    state_type: U16<LE>,
    unk0: I16<LE>,
    event_flag_id: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct EventDataPlatoonInfo {
    platoon_id_script_active: I32<LE>,
    state: I32<LE>,
    un8: I32<LE>,
    unkc: I32<LE>,
    group_part_indices: [I32<LE>; 32],
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct EventDataMount {
    rider_part_index: I32<LE>,
    mount_part_index: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct EventDataBirdTravelInfo {}

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct EventDataTalkInfo {
    pub unk0: I32<LE>,
    pub unk1: I32<LE>,
    pub unk2: I32<LE>,
    pub unk3: I32<LE>,
    pub unk4: I32<LE>,
    pub unk5: I32<LE>,
    pub unk6: I32<LE>,
    pub unk7: I32<LE>,
    pub unk8: I32<LE>,
    pub talk_id_1: I32<LE>,
    pub talk_id_2: I32<LE>,
    pub talk_id_3: I32<LE>,
    pub unk10: I32<LE>,
    pub unk11: I32<LE>,
    pub unk12: I32<LE>,
    pub unk13: I32<LE>,
    pub unk14: I32<LE>,
    pub unk15: I32<LE>,
    padding: Padding<56>,
}

impl Debug for EventDataTalkInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventDataTalkInfo")
            .field("unk0", &self.unk0.get())
            .field("unk1", &self.unk1.get())
            .field("unk2", &self.unk2.get())
            .field("unk3", &self.unk3.get())
            .field("unk4", &self.unk4.get())
            .field("unk5", &self.unk5.get())
            .field("unk6", &self.unk6.get())
            .field("unk7", &self.unk7.get())
            .field("unk8", &self.unk8.get())
            .field("talk_id_1", &self.talk_id_1.get())
            .field("talk_id_2", &self.talk_id_2.get())
            .field("talk_id_3", &self.talk_id_3.get())
            .field("unk10", &self.unk10.get())
            .field("unk11", &self.unk11.get())
            .field("unk12", &self.unk12.get())
            .field("unk13", &self.unk13.get())
            .field("unk14", &self.unk14.get())
            .field("unk15", &self.unk15.get())
            .finish()
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct EventDataGroupBattleInfo {
    pub event_flag_1: I32<LE>,
    pub enemy_entity_id_1: I32<LE>,
    pub enemy_entity_id_2: I32<LE>,
    pub event_flag_2: I32<LE>,
    pub enemy_entity_id_3: I32<LE>,
}
