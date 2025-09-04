use byteorder::LE;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, U32, U64};

use super::{MsbError, MsbParam, MsbVersion};
use crate::io_ext::read_wide_cstring;

#[derive(Debug)]
#[allow(unused, non_camel_case_types)]
pub struct MODEL_PARAM_ST<'a> {
    pub name: &'a WStr<LE>,
    model_type: U32<LE>,
    model_type_index: U32<LE>,
    sib_path: &'a WStr<LE>,
    instance_count: U32<LE>,
}

impl<'a> MsbParam<'a, MODEL_PARAM_ST<'a>, ModelType> for MODEL_PARAM_ST<'a> {
    const NAME: &'static str = "MODEL_PARAM_ST";

    fn read_entry(data: &'a [u8], _msb_version: &'a MsbVersion) -> Result<Self, MsbError> {
        let header = Header::ref_from_prefix(data).ok_or(MsbError::UnalignedValue)?;

        let name = read_wide_cstring(&data[header.name_offset.get() as usize..])?;
        let sib_path = read_wide_cstring(&data[header.sib_path_offset.get() as usize..])?;

        Ok(MODEL_PARAM_ST {
            name,
            sib_path,
            model_type: header.model_type,
            model_type_index: header.model_type_index,
            instance_count: header.instance_count,
        })
    }

    fn of_type(
        models: Result<impl Iterator<Item = Result<MODEL_PARAM_ST<'a>, MsbError>>, MsbError>,
        _model_type: ModelType,
    ) -> Vec<MODEL_PARAM_ST<'a>> {
        let mut models_of_type: Vec<MODEL_PARAM_ST<'a>> = vec![];

        if let Ok(models) = models {
            for model in models.flatten() {
                models_of_type.push(model);
            }
        }

        models_of_type
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn type_index(&self) -> u32 {
        self.model_type_index.get()
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    name_offset: U64<LE>,
    model_type: U32<LE>,
    model_type_index: U32<LE>,
    sib_path_offset: U64<LE>,
    instance_count: U32<LE>,
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum ModelType {
    // TODO: Determine different route types
}
