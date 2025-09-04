use byteorder::LE;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U16, U32, U64};

use super::MsbError;

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum PartType {
    MapPiece = 0,
    Enemy = 2,
    Player = 4,
    Collision = 5,
    DummyAsset = 9,
    DummyEnemy = 10,
    ConnectCollision = 11,
    Asset = 13,
    Unknown,
}

impl PartType {
    pub fn variants() -> Vec<(PartType, &'static str)> {
        vec![
            (PartType::MapPiece, "MapPiece"),
            (PartType::Enemy, "Enemy"),
            (PartType::Player, "Player"),
            (PartType::Collision, "Collision"),
            (PartType::DummyAsset, "DummyAsset"),
            (PartType::DummyEnemy, "DummyEnemy"),
            (PartType::ConnectCollision, "ConnectCollision"),
            (PartType::Asset, "Asset"),
        ]
    }
}

impl From<i32> for PartType {
    fn from(v: i32) -> Self {
        match v {
            0 => PartType::MapPiece,
            2 => PartType::Enemy,
            4 => PartType::Player,
            5 => PartType::Collision,
            9 => PartType::DummyAsset,
            10 => PartType::DummyEnemy,
            11 => PartType::ConnectCollision,
            13 => PartType::Asset,
            _ => PartType::Unknown,
        }
    }
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
    pub fn from_type_and_slice(part_type_id: i32, data: &'a [u8]) -> Result<Self, MsbError> {
        let part_type = PartType::from(part_type_id);
        Ok(match part_type {
            PartType::MapPiece => Self::MapPiece,
            PartType::Enemy => {
                Self::Enemy(PartDataEnemy::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }
            PartType::Player => {
                Self::Player(PartDataPlayer::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }
            PartType::Collision => Self::Collision(
                PartDataCollision::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            PartType::DummyAsset => Self::DummyAsset(
                PartDataDummyAsset::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            PartType::DummyEnemy => Self::DummyEnemy(
                PartDataEnemy::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            PartType::ConnectCollision => Self::ConnectCollision(
                PartDataConnectCollision::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),
            PartType::Asset => Self::Asset(PartDataAsset::from_slice(data)?),
            _ => return Err(MsbError::UnknownPartDataType(part_type_id)),
        })
    }
}

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct PartDataDummyEnemyUnk88 {
    unk0: I32<LE>,
    unk4: I16<LE>,
    unk6: I16<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PartDataPlayer {
    unk0: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
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
