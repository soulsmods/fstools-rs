use std::borrow::Cow;

use byteorder::LE;
use widestring::U16Str;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U16, U32, U64};

use super::{MsbError, MsbParam};
use crate::io_ext::{read_widestring, zerocopy::Padding};

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct PARTS_PARAM_ST<'a> {
    pub name: Cow<'a, U16Str>,
    pub id: U32<LE>,
    pub model_index: U32<LE>,
    pub sib: Cow<'a, U16Str>,
    pub position: [F32<LE>; 3],
    pub rotation: [F32<LE>; 3],
    pub scale: [F32<LE>; 3],
    pub map_layer: I32<LE>,
    pub masking_behavior: &'a MaskingBehavior,
    pub entity: &'a Entity,
    pub part: PartData<'a>,
    pub gparam: &'a Gparam,
    // TODO: represent the unk structures following the structures after
    // examining them with Ghidra.
}

impl<'a> MsbParam<'a> for PARTS_PARAM_ST<'a> {
    const NAME: &'static str = "PARTS_PARAM_ST";

    fn read_entry(data: &'a [u8]) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_widestring(&data[header.name_offset.get() as usize..])?;
        let sib = read_widestring(&data[header.sib_offset.get() as usize..])?;

        let masking_behavior = MaskingBehavior::ref_from_prefix(
            &data[header.masking_behavior_data_offset.get() as usize..],
        )
        .ok_or(MsbError::UnalignedValue)?;

        let entity = Entity::ref_from_prefix(&data[header.entity_data_offset.get() as usize..])
            .ok_or(MsbError::UnalignedValue)?;

        let part = PartData::from_type_and_slice(
            header.part_type.get(),
            &data[header.part_data_offset.get() as usize..],
        )?;

        let gparam = Gparam::ref_from_prefix(&data[header.gparam_data_offset.get() as usize..])
            .ok_or(MsbError::UnalignedValue)?;

        Ok(PARTS_PARAM_ST {
            name,
            id: header.id,
            model_index: header.model_index,
            sib,
            position: header.position,
            rotation: header.rotation,
            scale: header.scale,
            map_layer: header.map_layer,
            masking_behavior,
            entity,
            part,
            gparam,
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    unk8: U32<LE>,
    part_type: I32<LE>,
    id: U32<LE>,
    model_index: U32<LE>,
    sib_offset: U64<LE>,
    position: [F32<LE>; 3],
    rotation: [F32<LE>; 3],
    scale: [F32<LE>; 3],
    unk44: I32<LE>,
    map_layer: I32<LE>,
    _pad68: Padding<4>,
    masking_behavior_data_offset: U64<LE>,
    unk2_offset: U64<LE>,
    entity_data_offset: U64<LE>,
    part_data_offset: U64<LE>,
    gparam_data_offset: U64<LE>,
    scene_gparam_data_offset: U64<LE>,
    unk7_offset: U64<LE>,
    unk8_offset: U64<LE>,
    unk9_offset: U64<LE>,
    unk10_offset: U64<LE>,
    unk11_offset: U64<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
// Seems to be very oriented around masking behavior. Just called "PartUnk1" in
// soulstemplates.
pub struct MaskingBehavior {
    pub display_groups: [U32<LE>; 8],
    pub draw_groups: [U32<LE>; 8],
    pub collision_mask: [U32<LE>; 32],
    pub condition_1: u8,
    pub condition_2: u8,
    unkc2: u8,
    unkc3: u8,
    unkc4: I16<LE>,
    unkc6: U16<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Entity {
    entity_id: U32<LE>,
    unk4: u8,
    unk5: u8,
    unk6: u8,
    lantern: u8,
    pub lod_param: u8,
    unk9: u8,
    is_point_light_shadow_source: u8,
    unkb: u8,
    is_shadow_source: u8,
    is_static_shadow_source: u8,
    is_cascade_3_shadow_source: u8,
    unkf: u8,
    unk10: u8,
    is_shadow_destination: u8,
    is_shadow_only: u8,
    draw_by_reflect_cam: u8,
    draw_only_reflect_cam: u8,
    enable_on_above_shadow: u8,
    disable_point_light_effect: u8,
    unk17: u8,
    unk18: u8,
    unk19: u8,
    unk1a: u8,
    unk1b: u8,
    entity_groups: [U32<LE>; 8],
    unk3c: U16<LE>,
    unk3e: U16<LE>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum PartData<'a> {
    MapPiece,
    Enemy(&'a PartDataEnemy),
    Player(&'a PartDataPlayer),
    Collision(&'a PartDataCollision),
    DummyAsset(&'a PartDataDummyAsset),
    DummyEnemy(&'a PartDataEnemy),
    ConnectCollision(&'a PartDataConnectCollision),
    Asset(PartDataAsset),
}

impl<'a> PartData<'a> {
    pub fn from_type_and_slice(part_type: i32, data: &'a [u8]) -> Result<Self, MsbError> {
        Ok(match part_type {
            0 => Self::MapPiece,
            2 => Self::Enemy(PartDataEnemy::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),
            4 => {
                Self::Player(PartDataPlayer::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }
            5 => Self::Collision(
                PartDataCollision::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            9 => Self::DummyAsset(
                PartDataDummyAsset::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            10 => Self::DummyEnemy(
                PartDataEnemy::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            11 => Self::ConnectCollision(
                PartDataConnectCollision::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            13 => Self::Asset(PartDataAsset::from_slice(data)?),
            _ => return Err(MsbError::UnknownPartDataType(part_type)),
        })
    }
}

#[derive(FromZeroes, FromBytes)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataEnemy {
    unk0: U32<LE>,
    unk4: U32<LE>,
    think_param: U32<LE>,
    npc_param: U32<LE>,
    talk_id: U32<LE>,
    unk14: u8,
    unk15: u8,
    platoon: U16<LE>,
    chara_init: I32<LE>,
    collision_part_index: I32<LE>,
    unk20: U16<LE>,
    unk22: U16<LE>,
    unk24: I32<LE>,
    unk28: U32<LE>,
    unk2c: U32<LE>,
    unk30: U32<LE>,
    unk34: U32<LE>,
    backup_event_anim: I32<LE>,
    un3c: U32<LE>,
    unk40: U32<LE>,
    unk44: U32<LE>,
    unk48: U32<LE>,
    unk4c: U32<LE>,
    unk50: U32<LE>,
    unk54: U32<LE>,
    unk58: U32<LE>,
    unk5c: U32<LE>,
    unk60: U32<LE>,
    unk64: U32<LE>,
    unk68: U32<LE>,
    unk6c: U32<LE>,
    unk70: U32<LE>,
    unk74: U32<LE>,
    unk78: U64<LE>,
    unk80: U32<LE>,
    unk84: F32<LE>,
    unk88: [PartDataDummyEnemyUnk88; 5],
}

impl std::fmt::Debug for PartDataEnemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartDataEnemy")
            .field("think_param", &self.think_param.get())
            .field("npc_param", &self.npc_param.get())
            .field("talk_id", &self.talk_id.get())
            .field("platoon", &self.platoon.get())
            .field("chara_init", &self.chara_init.get())
            .field("platoon", &self.platoon.get())
            .field("collision_part_index", &self.collision_part_index.get())
            .field("backup_event_anim", &self.backup_event_anim.get())
            .finish()
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataDummyEnemyUnk88 {
    unk0: I32<LE>,
    unk4: I16<LE>,
    unk6: I16<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataPlayer {
    unk0: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataCollision {
    unk0: u8,
    unk1: i8,
    unk2: i8,
    unk3: u8,
    unk4: F32<LE>,
    unk8: U32<LE>,
    unkc: U32<LE>,
    unk10: U32<LE>,
    unk14: F32<LE>,
    unk18: I32<LE>,
    unk1c: I32<LE>,
    play_region: I32<LE>,
    unk24: I16<LE>,
    unk26: U16<LE>,
    unk28: I32<LE>,
    unk2c: I32<LE>,
    unk30: I32<LE>,
    unk34: u8,
    unk35: i8,
    unk36: u8,
    unk37: u8,
    unk38: I32<LE>,
    unk3c: I16<LE>,
    unk3e: I16<LE>,
    unk40: F32<LE>,
    unk44: U32<LE>,
    unk48: U32<LE>,
    unk4c: I16<LE>,
    unk4e: I16<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataDummyAsset {
    unk0: I32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    unk14: I32<LE>,
    unk18: I32<LE>,
    unk1c: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataConnectCollision {
    collision_index: U32<LE>,
    map_id: [i8; 4],
    unk8: u8,
    unk9: u8,
    unka: i8,
    unkb: u8,
}

#[derive(Debug)]
#[allow(unused)]
pub struct PartDataAsset {
    // TODO: do the rest of the format
}

impl PartDataAsset {
    fn from_slice(data: &[u8]) -> Result<Self, MsbError> {
        let _header = PartDataAssetHeader::ref_from_suffix(data).ok_or(MsbError::UnalignedValue);

        Ok(Self {})
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PartDataAssetHeader {
    unk0: U16<LE>,
    unk2: U16<LE>,
    unk4: U32<LE>,
    unk8: U32<LE>,
    unkc: U32<LE>,
    unk10: u8,
    unk11: u8,
    unk12: i8,
    unk13: u8,
    unk14: U32<LE>,
    unk18: U32<LE>,
    unk1c: I16<LE>,
    unk1e: I16<LE>,
    unk20: I32<LE>,
    unk24: I32<LE>,
    unk28: U32<LE>,
    unk2c: U32<LE>,
    unk30: I32<LE>,
    unk34: I32<LE>,
    unk38: [I32<LE>; 6],
    unk50: u8,
    unk51: u8,
    unk52: u8,
    unk53: u8,
    unk54: I32<LE>,
    unk58: I32<LE>,
    unk5c: I32<LE>,
    unk60: I32<LE>,
    unk64: I32<LE>,
    unk68_offset: U64<LE>,
    unk70_offset: U64<LE>,
    unk78_offset: U64<LE>,
    unk80_offset: U64<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Gparam {
    light_set: I32<LE>,
    fog_param: I32<LE>,
    light_scattering: U32<LE>,
    environment_map: U32<LE>,
}
