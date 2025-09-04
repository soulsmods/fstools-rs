use byteorder::LE;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U32};

use super::MsbError;

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum PointType {
    Other = -1,
    EnvironmentMapPoint = 2,
    RespawnPoint = 3,
    Sound = 4,
    Sfx = 5,
    WindSfx = 6,
    SpawnPoint = 8,
    EnvironmentMapEffectBox = 17,
    Connection = 21,
    MufflingBox = 28,
    MufflingPortal = 29,
    SoundRegion = 30,
    PatrolRoute = 32,
    MapPoint = 33,
    WeatherOverride = 35,
    GroupDefeatReward = 37,
    Hitset = 40,
    WeatherCreateAssetPoint = 42,
    EnvironmentMapOutput = 44,
    MountJump = 46,
    Dummy = 48,
    FallPreventionRemoval = 49,
    MapAttachPoint = 54,
    BirdTravelRoute = 55,
    ClearPersonInfoPoint = 56,
    SuddenDeathArea = 57,
    UserEdgeEliminationInterior = 58,
    UserEdgeEliminationExterior = 59,
    Unknown,
}

impl PointType {
    pub fn variants() -> Vec<(PointType, &'static str)> {
        vec![
            (PointType::Other, "Other"),
            (PointType::EnvironmentMapPoint, "EnvironmentMapPoint"),
            (PointType::RespawnPoint, "RespawnPoint"),
            (PointType::Sound, "Sound"),
            (PointType::Sfx, "Sfx"),
            (PointType::WindSfx, "WindSfx"),
            (PointType::SpawnPoint, "SpawnPoint"),
            (
                PointType::EnvironmentMapEffectBox,
                "EnvironmentMapEffectBox",
            ),
            (PointType::Connection, "Connection"),
            (PointType::MufflingBox, "MufflingBox"),
            (PointType::MufflingPortal, "MufflingPortal"),
            (PointType::SoundRegion, "SoundRegion"),
            (PointType::PatrolRoute, "PatrolRoute"),
            (PointType::MapPoint, "MapPoint"),
            (PointType::WeatherOverride, "WeatherOverride"),
            (PointType::GroupDefeatReward, "GroupDefeatReward"),
            (PointType::Hitset, "Hitset"),
            (
                PointType::WeatherCreateAssetPoint,
                "WeatherCreateAssetPoint",
            ),
            (PointType::EnvironmentMapOutput, "EnvironmentMapOutput"),
            (PointType::MountJump, "MountJump"),
            (PointType::Dummy, "Dummy"),
            (PointType::FallPreventionRemoval, "FallPreventionRemoval"),
            (PointType::MapAttachPoint, "MapAttachPoint"),
            (PointType::BirdTravelRoute, "BirdTravelRoute"),
            (PointType::ClearPersonInfoPoint, "ClearPersonInfoPoint"),
            (PointType::SuddenDeathArea, "SuddenDeathArea"),
            (
                PointType::UserEdgeEliminationInterior,
                "UserEdgeEliminationInterior",
            ),
            (
                PointType::UserEdgeEliminationExterior,
                "UserEdgeEliminationExterior",
            ),
        ]
    }
}

impl From<i32> for PointType {
    fn from(v: i32) -> Self {
        match v {
            -1 => PointType::Other,
            2 => PointType::EnvironmentMapPoint,
            3 => PointType::RespawnPoint,
            4 => PointType::Sound,
            5 => PointType::Sfx,
            6 => PointType::WindSfx,
            8 => PointType::SpawnPoint,
            17 => PointType::EnvironmentMapEffectBox,
            21 => PointType::Connection,
            28 => PointType::MufflingBox,
            29 => PointType::MufflingPortal,
            30 => PointType::SoundRegion,
            32 => PointType::PatrolRoute,
            33 => PointType::MapPoint,
            35 => PointType::WeatherOverride,
            37 => PointType::GroupDefeatReward,
            40 => PointType::Hitset,
            42 => PointType::WeatherCreateAssetPoint,
            44 => PointType::EnvironmentMapOutput,
            46 => PointType::MountJump,
            48 => PointType::Dummy,
            49 => PointType::FallPreventionRemoval,
            54 => PointType::MapAttachPoint,
            55 => PointType::BirdTravelRoute,
            56 => PointType::ClearPersonInfoPoint,
            57 => PointType::SuddenDeathArea,
            58 => PointType::UserEdgeEliminationInterior,
            59 => PointType::UserEdgeEliminationExterior,
            _ => PointType::Unknown,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub enum PointData<'a> {
    Other,
    EnvironmentMapPoint(&'a PointDataEnvironmentMapPoint),
    RespawnPoint(&'a PointDataRespawnPoint),
    Sound(&'a PointDataSound),
    Sfx(&'a PointDataSfx),
    WindSfx(&'a PointDataWindSfx),
    SpawnPoint(&'a PointDataSpawnPoint),
    EnvironmentMapEffectBox(&'a PointDataEnvironmentMapEffectBox),
    Connection(&'a PointDataConnection),
    MufflingBox(&'a PointDataMufflingBox),
    MufflingPortal(&'a PointDataMufflingPortal),
    SoundRegion(&'a PointDataSoundRegion),
    PatrolRoute(&'a PointDataPatrolRoute),
    MapPoint(&'a PointDataMapPoint),
    WeatherOverride(&'a PointDataWeatherOverride),
    GroupDefeatReward(&'a PointDataGroupDefeatReward),
    Hitset(&'a PointDataHitset),
    WeatherCreateAssetPoint(&'a PointDataWeatherCreateAssetPoint),
    EnvironmentMapOutput(&'a PointDataEnvironmentMapOutput),
    MountJump(&'a PointDataMountJump),
    Dummy(&'a PointDataDummy),
    FallPreventionRemoval(&'a PointDataFallPreventionRemoval),
    MapAttachPoint(&'a PointDataMapAttachPoint),
    BirdTravelRoute(&'a PointDataBirdTravelRoute),
    ClearPersonInfoPoint(&'a PointDataClearPersonInfoPoint),
    SuddenDeathArea(&'a PointDataSuddenDeathArea),
    UserEdgeEliminationInterior(&'a PointDataUserEdgeEliminationInterior),
    UserEdgeEliminationExterior(&'a PointDataUserEdgeEliminationExterior),
}

impl<'a> PointData<'a> {
    pub fn from_type_and_slice(point_type_id: i32, data: &'a [u8]) -> Result<Self, MsbError> {
        let point_type = PointType::from(point_type_id);
        Ok(match point_type {
            PointType::Other => Self::Other,

            PointType::EnvironmentMapPoint => Self::EnvironmentMapPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::RespawnPoint => Self::RespawnPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::Sound => {
                Self::Sound(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::Sfx => {
                Self::Sfx(PointDataSfx::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::WindSfx => {
                Self::WindSfx(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::SpawnPoint => {
                Self::SpawnPoint(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::EnvironmentMapEffectBox => Self::EnvironmentMapEffectBox(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::Connection => {
                Self::Connection(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::MufflingBox => {
                Self::MufflingBox(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::MufflingPortal => Self::MufflingPortal(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::SoundRegion => {
                Self::SoundRegion(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::PatrolRoute => {
                Self::PatrolRoute(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::MapPoint => {
                Self::MapPoint(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::WeatherOverride => Self::WeatherOverride(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::GroupDefeatReward => Self::GroupDefeatReward(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::Hitset => {
                Self::Hitset(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::WeatherCreateAssetPoint => Self::WeatherCreateAssetPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::EnvironmentMapOutput => Self::EnvironmentMapOutput(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::MountJump => {
                Self::MountJump(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::Dummy => {
                Self::Dummy(FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?)
            }

            PointType::FallPreventionRemoval => Self::FallPreventionRemoval(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::MapAttachPoint => Self::MapAttachPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::BirdTravelRoute => Self::BirdTravelRoute(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::ClearPersonInfoPoint => Self::ClearPersonInfoPoint(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::SuddenDeathArea => Self::SuddenDeathArea(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::UserEdgeEliminationInterior => Self::UserEdgeEliminationInterior(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            PointType::UserEdgeEliminationExterior => Self::UserEdgeEliminationExterior(
                FromBytes::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?,
            ),

            _ => return Err(MsbError::UnknownPointDataType(point_type_id)),
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataRespawnPoint {
    // TODO: Examine struct layout in Ghidra
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataSfx {
    effect_id: U32<LE>,
    unk4: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataWindSfx {
    effect_id: U32<LE>,
    wind_area_index: U32<LE>,
    // Seems to be some form of bit set?
    unk8: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataSpawnPoint {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataConnection {
    map_id: [u8; 4],
    unk4: I32<LE>,
    unk8: I32<LE>,
    unkc: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataHitset {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataPatrolRoute {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
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
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataWeatherCreateAssetPoint {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataEnvironmentMapOutput {
    unk0: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataMountJump {
    unk0: I32<LE>,
    unk4: F32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataDummy {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataFallPreventionRemoval {
    unk0: I32<LE>,
    unk4: I32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataMapAttachPoint {
    // TODO: Examine struct layout in Ghidra
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataBirdTravelRoute {
    // TODO: Examine struct layout in Ghidra
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataClearPersonInfoPoint {
    // TODO: Examine struct layout in Ghidra
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataSuddenDeathArea {
    // TODO: Examine struct layout in Ghidra
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataUserEdgeEliminationInterior {
    // TODO: Examine struct layout in Ghidra
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct PointDataUserEdgeEliminationExterior {
    // TODO: Examine struct layout in Ghidra
}
