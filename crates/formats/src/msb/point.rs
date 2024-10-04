

use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U32, U64};

use super::{MsbError, MsbParam};
use crate::io_ext::read_wide_cstring;

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct POINT_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    pub id: U32<LE>,
    pub shape_type: U32<LE>,
    pub position: [F32<LE>; 3],
    pub rotation: [F32<LE>; 3],
    pub point: PointData<'a>,
}

impl<'a> MsbParam<'a> for POINT_PARAM_ST<'a> {
    const NAME: &'static str = "POINT_PARAM_ST";

    fn read_entry(data: &'a [u8]) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[header.name_offset.get() as usize..])?;

        let point = PointData::from_type_and_slice(
            header.point_type.get(),
            &data[header.point_data_offset.get() as usize..],
        )?;

        Ok(POINT_PARAM_ST {
            name,
            id: header.id,
            shape_type: header.shape_type,
            position: header.position,
            rotation: header.rotation,
            point,
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    point_type: I32<LE>,
    id: U32<LE>,
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

#[derive(Debug)]
#[allow(unused)]
pub enum PointData<'a> {
    Other,
    InvasionPoint(&'a PointDataInvasionPoint),
    EnvironmentMapPoint(&'a PointDataEnvironmentMapPoint),
    Sound(&'a PointDataSound),
    Sfx(&'a PointDataSfx),
    WindSfx(&'a PointDataWindSfx),
    SpawnPoint(&'a PointDataSpawnPoint),
    Message(&'a PointDataMessage),
    EnvironmentMapEffectBox(&'a PointDataEnvironmentMapEffectBox),
    WindArea,
    Connection(&'a PointDataConnection),
    Hitset(&'a PointDataHitset),
    PatrolRoute22(&'a PointDataPatrolRoute22),
    BuddySummonPoint(&'a PointDataBuddySummonPoint),
    MufflingBox(&'a PointDataMufflingBox),
    MufflingPortal(&'a PointDataMufflingPortal),
    SoundRegion(&'a PointDataSoundRegion),
    PatrolRoute(&'a PointDataPatrolRoute),
    MapPoint(&'a PointDataMapPoint),
    WeatherOverride(&'a PointDataWeatherOverride),
    AutoDrawGroupPoint(&'a PointDataAutoDrawGroupPoint),
    GroupDefeatReward(&'a PointDataGroupDefeatReward),
    MapPointDiscoveryOverride,
    MapPointParticipationOverride,
    NpcArea(&'a PointDataNpcArea),
    WeatherCreateAssetPoint(&'a PointDataWeatherCreateAssetPoint),
    PlayArea(&'a PointDataPlayArea),
    EnvironmentMapOutput(&'a PointDataEnvironmentMapOutput),
    MountJump(&'a PointDataMountJump),
    Dummy(&'a PointDataDummy),
    FallPreventionRemoval(&'a PointDataFallPreventionRemoval),
    NavmeshCutting(&'a PointDataNavmeshCutting),
    MapNameOverride(&'a PointDataMapNameOverride),
    MountJumpFall(&'a PointDataMountJumpFall),
    HorseProhibition(&'a PointDataHorseProhibition),
}

impl<'a> PointData<'a> {
    fn from_type_and_slice(point_type: i32, data: &'a [u8]) -> Result<Self, MsbError> {
        Ok(match point_type {
            -1 => Self::Other,

            1 => Self::InvasionPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            2 => Self::EnvironmentMapPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            4 => Self::Sound(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            5 => Self::Sfx(PointDataSfx::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            6 => Self::WindSfx(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            8 => {
                Self::SpawnPoint(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            9 => Self::Message(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            17 => Self::EnvironmentMapEffectBox(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            18 => Self::WindArea,

            21 => {
                Self::Connection(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            22 => Self::PatrolRoute22(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            26 => Self::BuddySummonPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            28 => {
                Self::MufflingBox(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            29 => Self::MufflingPortal(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            30 => {
                Self::SoundRegion(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            32 => {
                Self::PatrolRoute(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            33 => Self::MapPoint(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            35 => Self::WeatherOverride(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            36 => Self::AutoDrawGroupPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            37 => Self::GroupDefeatReward(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            38 => Self::MapPointDiscoveryOverride,

            39 => Self::MapPointParticipationOverride,

            40 => Self::Hitset(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            41 => Self::NpcArea(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            42 => Self::WeatherCreateAssetPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            43 => Self::PlayArea(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            44 => Self::EnvironmentMapOutput(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            46 => {
                Self::MountJump(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            48 => Self::Dummy(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?),

            49 => Self::FallPreventionRemoval(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            50 => Self::NavmeshCutting(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            51 => Self::MapNameOverride(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            52 => Self::MountJumpFall(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            53 => Self::HorseProhibition(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            _ => return Err(MsbError::UnknownPointDataType(point_type)),
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataInvasionPoint {
    priority: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataEnvironmentMapPoint {
    unk0: F32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: u8,
    unkd: u8,
    unke: u8,
    unkf: u8,
    unk10: F32<LE>,
    unk14: F32<LE>,
    map_id: [u8; 4],
    unk1c: U32<LE>,
    unk20: U32<LE>,
    unk24: U32<LE>,
    unk28: U32<LE>,
    unk2c: u8,
    unk2d: u8,
    unk2e: u8,
    unk2f: u8,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataSound {
    sound_type: U32<LE>,
    sound_id: U32<LE>,
    child_point_indices: [I32<LE>; 16],
    unk48: u8,
    unk49: u8,
    unk4a: u8,
    unk4b: u8,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataSfx {
    effect_id: U32<LE>,
    unk4: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataWindSfx {
    effect_id: U32<LE>,
    wind_area_index: U32<LE>,
    // Seems to be some form of bit set?
    unk8: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataSpawnPoint {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMessage {
    pub message_id: I16<LE>,
    pub unk2: I16<LE>,
    // Seems to always be true/false? Could be a single byte with some padding?
    pub hidden: U32<LE>,
    pub item_lot: I32<LE>,
    pub unkc: U32<LE>,
    pub event_flag: I32<LE>,
    pub character_model_id: I32<LE>,
    pub npc_param_id: I32<LE>,
    pub animation_id: I32<LE>,
    pub chara_init_param_id: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataEnvironmentMapEffectBox {
    unk00: F32<LE>,
    compare: F32<LE>,
    unk8: u8,
    unk9: u8,
    unka: I16<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    unk14: I32<LE>,
    unk18: I32<LE>,
    unk1c: I32<LE>,
    unk20: I32<LE>,
    unk24: F32<LE>,
    unk28: F32<LE>,
    unk2c: I16<LE>,
    unk2e: u8,
    unk2f: u8,
    unk30: I16<LE>,
    unk32: u8,
    unk33: u8,
    unk34: I16<LE>,
    unk36: I16<LE>,
    unk38: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataConnection {
    map_id: [u8; 4],
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataHitset {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataPatrolRoute22 {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataBuddySummonPoint {
    unk0: I32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMufflingBox {
    unk0: I32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    unk14: I32<LE>,
    unk18: I32<LE>,
    unk1c: I32<LE>,
    unk20: I32<LE>,
    unk24: F32<LE>,
    unk28: I32<LE>,
    unk2c: I32<LE>,
    unk30: I32<LE>,
    unk34: F32<LE>,
    unk38: I32<LE>,
    unk3c: F32<LE>,
    unk40: F32<LE>,
    unk44: F32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMufflingPortal {
    unk0: I32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    unk14: I32<LE>,
    unk18: I32<LE>,
    unk1c: I32<LE>,
    unk20: I32<LE>,
    unk24: I32<LE>,
    unk28: I32<LE>,
    unk2c: I32<LE>,
    unk30: I32<LE>,
    unk34: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataSoundRegion {
    unk0: i8,
    unk1: i8,
    unk2: i8,
    unk3: i8,
    unk4: I32<LE>,
    unk8: I16<LE>,
    unka: I16<LE>,
    unkc: u8,
    unkd: u8,
    unke: u8,
    unkf: u8,
    unk10: I32<LE>,
    unk14: I32<LE>,
    unk18: I32<LE>,
    unk1c: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataPatrolRoute {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMapPoint {
    world_map_point_param: I32<LE>,
    unk4: I32<LE>,
    unk8: F32<LE>,
    unkc: F32<LE>,
    unk10: I32<LE>,
    unk14: F32<LE>,
    unk18: F32<LE>,
    unk1c: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataWeatherOverride {
    weather_lot_param: I32<LE>,
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
pub struct PointDataAutoDrawGroupPoint {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataGroupDefeatReward {
    unk0: I32<LE>,
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
    unk10: I32<LE>,
    unk14: [I32<LE>; 8],
    unk34: I32<LE>,
    unk38: I32<LE>,
    unk3c: I32<LE>,
    unk40: I32<LE>,
    unk44: I32<LE>,
    unk48: I32<LE>,
    unk4c: I32<LE>,
    unk50: I32<LE>,
    unk54: I32<LE>,
    unk58: I32<LE>,
    unk5c: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataNpcArea {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataWeatherCreateAssetPoint {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataPlayArea {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataEnvironmentMapOutput {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMountJump {
    unk0: I32<LE>,
    unk4: F32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataDummy {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataFallPreventionRemoval {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataNavmeshCutting {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMapNameOverride {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataMountJumpFall {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct PointDataHorseProhibition {
    unk0: I32<LE>,
    unk4: I32<LE>,
}
