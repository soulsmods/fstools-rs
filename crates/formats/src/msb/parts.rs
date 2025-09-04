pub mod elden_ring;
pub mod nightreign;

use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, F32, I16, I32, U16, U32, U64};

use super::{MsbError, MsbParam, MsbVersion};
use crate::{
    io_ext::{read_wide_cstring, zerocopy::Padding},
    msb::parts::PartData::{EldenRing, Nightreign},
};

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct PARTS_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    pub model_index: U32<LE>,
    pub sib: &'a WStr<LE>,
    pub position: [F32<LE>; 3],
    pub rotation: [F32<LE>; 3],
    pub scale: [F32<LE>; 3],
    pub map_layer: I32<LE>,
    pub masking_behavior: &'a MaskingBehavior,
    pub entity: &'a Entity,
    pub part_type: (I32<LE>, PartType),
    pub part_type_index: U32<LE>,
    pub part: PartData<'a>,
    pub gparam: &'a Gparam,
    // TODO: represent the unk structures following the structures after
    // examining them with Ghidra.
}

impl<'a> MsbParam<'a, PARTS_PARAM_ST<'a>, PartType> for PARTS_PARAM_ST<'a> {
    const NAME: &'static str = "PARTS_PARAM_ST";

    fn read_entry(data: &'a [u8], version: &'a MsbVersion) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[header.name_offset.get() as usize..])?;
        let sib = read_wide_cstring(&data[header.sib_offset.get() as usize..])?;

        let masking_behavior = MaskingBehavior::ref_from_prefix(
            &data[header.masking_behavior_data_offset.get() as usize..],
        )
        .ok_or(MsbError::UnalignedValue)?;

        let entity = Entity::ref_from_prefix(&data[header.entity_data_offset.get() as usize..])
            .ok_or(MsbError::UnalignedValue)?;

        let part_type: PartType;
        let part: PartData;

        match version {
            MsbVersion::EldenRing => {
                part_type = PartType::EldenRing(elden_ring::PartType::from(header.part_type.get()));
                part = EldenRing(elden_ring::PartData::from_type_and_slice(
                    header.part_type.get(),
                    &data[header.part_data_offset.get() as usize..],
                )?);
            }
            MsbVersion::Nightreign => {
                part_type =
                    PartType::Nightreign(nightreign::PartType::from(header.part_type.get()));
                part = Nightreign(nightreign::PartData::from_type_and_slice(
                    header.part_type.get(),
                    &data[header.part_data_offset.get() as usize..],
                )?);
            }
        };

        let gparam = Gparam::ref_from_prefix(&data[header.gparam_data_offset.get() as usize..])
            .ok_or(MsbError::UnalignedValue)?;

        Ok(PARTS_PARAM_ST {
            name,
            model_index: header.model_index,
            sib,
            position: header.position,
            rotation: header.rotation,
            scale: header.scale,
            map_layer: header.map_layer,
            masking_behavior,
            entity,
            part_type: (header.part_type, part_type),
            part_type_index: header.part_type_index,
            part,
            gparam,
        })
    }

    fn of_type(
        parts: Result<impl Iterator<Item = Result<PARTS_PARAM_ST<'a>, MsbError>>, MsbError>,
        part_type: PartType,
    ) -> Vec<PARTS_PARAM_ST<'a>> {
        let mut parts_of_type: Vec<PARTS_PARAM_ST<'a>> = vec![];

        if let Ok(parts) = parts {
            for part in parts.flatten() {
                if part.part_type.1 == part_type {
                    parts_of_type.push(part);
                }
            }
        }

        parts_of_type
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn type_index(&self) -> u32 {
        self.part_type_index.get()
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    unk8: U32<LE>,
    part_type: I32<LE>,
    part_type_index: U32<LE>,
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
#[repr(C, packed)]
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
#[repr(C, packed)]
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

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum PartType {
    EldenRing(elden_ring::PartType),
    Nightreign(nightreign::PartType),
}

#[derive(Debug)]
#[allow(unused)]
pub enum PartData<'a> {
    EldenRing(elden_ring::PartData<'a>),
    Nightreign(nightreign::PartData<'a>),
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Gparam {
    light_set: I32<LE>,
    fog_param: I32<LE>,
    light_scattering: U32<LE>,
    environment_map: U32<LE>,
}
