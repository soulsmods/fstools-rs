use std::io;

use byteorder::LE;
use thiserror::Error;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, Ref, F32, I32, U32, U64};

use crate::io_ext::{read_wide_cstring, zerocopy::Padding, ReadWidestringError};

#[derive(Debug, Error)]
pub enum MetaparamError {
    #[error("Could not copy bytes {0}")]
    Io(#[from] io::Error),

    #[error("Could not read string")]
    String(#[from] ReadWidestringError),

    #[error("Not a metaparam (expected SMD\\0 magic)")]
    Magic,

    #[error("Unsupported metaparam version {0}")]
    Version(u32),

    #[error("A table is out of bounds or misaligned")]
    Table,
}

pub struct Metaparam<'a> {
    bytes: &'a [u8],
    header: &'a Header,
    textures: &'a [Texture],
    render_passes: &'a [RenderPassEntry],
    ca_params: &'a [CaParam],
    mtd_params: &'a [MtdParam],
}

impl<'a> Metaparam<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, MetaparamError> {
        let (header, next) =
            Ref::<_, Header>::new_from_prefix(bytes).ok_or(MetaparamError::Table)?;
        if &header.magic != b"SMD\0" {
            return Err(MetaparamError::Magic);
        }
        if header.version.get() != 6 {
            return Err(MetaparamError::Version(header.version.get()));
        }

        let (textures, next) =
            Texture::slice_from_prefix(next, header.texture_count.get() as usize)
                .ok_or(MetaparamError::Table)?;
        let (render_pass_config, _) =
            Ref::<_, RenderPassConfig>::new_from_prefix(next).ok_or(MetaparamError::Table)?;

        let table = |offset: u64| bytes.get(offset as usize..).ok_or(MetaparamError::Table);
        let render_passes = RenderPassEntry::slice_from_prefix(
            table(render_pass_config.entries_offset.get())?,
            render_pass_config.entry_count.get() as usize,
        )
        .ok_or(MetaparamError::Table)?
        .0;
        let ca_params = CaParam::slice_from_prefix(
            table(header.ca_params_offset.get())?,
            header.ca_param_count.get() as usize,
        )
        .ok_or(MetaparamError::Table)?
        .0;
        let mtd_params = MtdParam::slice_from_prefix(
            table(header.mtd_params_offset.get())?,
            header.mtd_param_count.get() as usize,
        )
        .ok_or(MetaparamError::Table)?
        .0;

        Ok(Self {
            bytes,
            header: header.into_ref(),
            textures,
            render_passes,
            ca_params,
            mtd_params,
        })
    }

    /// Size in bytes of the constant buffer the parameters are laid out in.
    pub fn cbuffer_size(&self) -> u32 {
        self.header.cbuffer_size.get()
    }

    /// The shader program's ID.
    pub fn shader_id(&self) -> u32 {
        self.header.shader_id.get()
    }

    pub fn flags(&self) -> u32 {
        self.header.flags.get()
    }

    pub fn textures(&self) -> impl Iterator<Item = Result<TextureIterElement<'_>, MetaparamError>> {
        self.textures.iter().map(|e| {
            Ok(TextureIterElement {
                name: self.string(e.name_offset.get())?,
                default_texture_path: self.string(e.default_texture_path_offset.get())?,
                uv_group_name: self.string(e.uv_group_name_offset.get())?,
                slot: e.slot,
                texture_type: e.texture_type,
            })
        })
    }

    /// The samplers each render pass binds.
    pub fn render_passes(&self) -> &[RenderPassEntry] {
        self.render_passes
    }

    pub fn ca_params(
        &self,
    ) -> impl Iterator<Item = Result<CaParamIterElement<'_>, MetaparamError>> {
        self.ca_params.iter().map(|e| {
            Ok(CaParamIterElement {
                name: self.string(e.name_offset.get())?,
                sequence_index: e.sequence_index.get(),
                param_id: e.param_id.get(),
                component_index: e.component_index.get(),
                default_value: e.default_value.get(),
                flags: e.flags.get(),
                param_key: e.param_key.get(),
            })
        })
    }

    pub fn mtd_params(
        &self,
    ) -> impl Iterator<Item = Result<MtdParamIterElement<'_>, MetaparamError>> {
        self.mtd_params.iter().map(|e| {
            Ok(MtdParamIterElement {
                name: self.string(e.name_offset.get())?,
                start_offset: e.start_offset.get(),
                sequence_index: e.sequence_index.get(),
                value_type: e.value_type,
                is_shader_input: e.not_shader_input != 2,
                is_srgb: e.is_srgb != 0,
                param_key: e.param_key.get(),
                default_value: e.default_value.map(|v| v.get()),
            })
        })
    }

    fn string(&self, offset: u64) -> Result<&'a WStr<LE>, MetaparamError> {
        let bytes = self
            .bytes
            .get(offset as usize..)
            .ok_or(MetaparamError::Table)?;

        Ok(read_wide_cstring(bytes)?)
    }
}

impl<'a> std::fmt::Debug for Metaparam<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metaparam")
            .field("header", self.header)
            .field("textures", &self.textures)
            .field("render_passes", &self.render_passes)
            .field("ca_params", &self.ca_params)
            .field("mtd_params", &self.mtd_params)
            .finish()
    }
}

pub struct TextureIterElement<'a> {
    /// The shader's name for the texture, e.g. `M_AMSN__snp_Texture2D_0_AlbedoMap_0`. A MATBIN
    /// sampler of the same name sets it.
    pub name: &'a WStr<LE>,

    /// Texture used when the MATBIN leaves the sampler unset. Empty if none.
    pub default_texture_path: &'a WStr<LE>,

    /// The texture's UV group, e.g. `group_1`, whose `<group>_CommonUV-UVParam` MTD parameter
    /// scales its texture coordinates.
    pub uv_group_name: &'a WStr<LE>,

    /// The shader's texture register, and its index into `FC_TextureTilingScale`.
    pub slot: u8,

    /// How this texture is used in the material system, e.g. albedo, metallic, vector
    pub texture_type: u8,
}

pub struct CaParamIterElement<'a> {
    /// `<parameter>_<component>`. A MATBIN sets the parameter, without the component.
    pub name: &'a WStr<LE>,

    /// Order in the `SAT_CAParam` block: each read component takes the next float.
    pub sequence_index: i32,

    /// GXMD parameter ID, which FLVER GX items set it by.
    pub param_id: i32,

    /// Which component of its parameter it is. Component 4 of a color is its intensity, which
    /// the shader doesn't read.
    pub component_index: i32,
    pub default_value: f32,

    /// Its upper 16 bits mark components the shader doesn't read, such as `_overwrite` params.
    pub flags: i32,
    pub param_key: i32,
}

impl CaParamIterElement<'_> {
    /// Whether the shader reads this component
    pub fn is_shader_input(&self) -> bool {
        self.flags as u32 & 0xffff_0000 == 0 && self.component_index != 4
    }
}

pub struct MtdParamIterElement<'a> {
    /// The name a MATBIN sets it by.
    pub name: &'a WStr<LE>,

    /// Where it starts in `cbMtdParam`, in 4 byte words.
    pub start_offset: i32,
    pub sequence_index: i32,
    pub value_type: u8,

    /// Whether the shader reads it, rather than only the renderer.
    pub is_shader_input: bool,

    /// Whether it's a color authored in sRGB, which the shader reads as linear.
    pub is_srgb: bool,

    pub param_key: u32,
    pub default_value: [f32; 5],
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    magic: [u8; 4],
    unk04: U32<LE>,
    version: U32<LE>,
    texture_count: U32<LE>,

    /// Seems to always equal `ca_params_offset`.
    unk_offset: U64<LE>,
    ca_params_offset: U64<LE>,
    mtd_params_offset: U64<LE>,
    ca_param_count: U32<LE>,
    mtd_param_count: U32<LE>,

    /// Always 0x0401 across all ELDEN RING metaparams.
    unk30: U32<LE>,
    cbuffer_size: U32<LE>,
    flags: U32<LE>,
    unk3c: U32<LE>,
    unk40: U32<LE>,
    unk44: U32<LE>,
    gxmd_key: U32<LE>,
    ca_param_block_size: U32<LE>,
    unk50: U32<LE>,
    /// Same as `ca_param_count`.
    ca_param_count2: U32<LE>,
    shader_id: U32<LE>,
    _padding5c: Padding<0x3c>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Texture {
    name_offset: U64<LE>,
    unk08: u8,
    slot: u8,
    unk0a: u8,
    texture_type: u8,
    unk0c: I32<LE>,
    default_texture_path_offset: U64<LE>,
    uv_group_name_offset: U64<LE>,
    _padding20: Padding<0x10>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct RenderPassConfig {
    entries_offset: U64<LE>,
    entry_count: U32<LE>,
    _padding0c: Padding<0x1c>,
}

/// The samplers a render pass binds.
#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct RenderPassEntry {
    /// Which pass. Unknown what each corresponds to.
    pub pass_id: I32<LE>,
    unk04: U32<LE>,
    /// Bitmask of the texture slots bound to pixel shaders.
    pub sampler_mask: U32<LE>,
    /// Bitmask of the texture slots bound to hull shaders.
    pub tessellation_sampler_mask: U32<LE>,
    /// Bitmask of the texture slots bound to vertex shaders.
    pub vertex_sampler_mask: U32<LE>,
    unk14: U32<LE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct CaParam {
    name_offset: U64<LE>,
    _padding08: Padding<0x1c>,
    unk24: U32<LE>,
    sequence_index: I32<LE>,
    param_id: I32<LE>,
    component_index: I32<LE>,
    default_value: F32<LE>,
    flags: I32<LE>,
    param_key: I32<LE>,
    unk40: U32<LE>,
    unk44: U32<LE>,
    _padding48: Padding<0x18>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct MtdParam {
    name_offset: U64<LE>,
    _padding08: Padding<0x1c>,
    start_offset: I32<LE>,
    sequence_index: I32<LE>,
    value_type: u8,

    /// 2 when only the renderer reads it, else 0.
    not_shader_input: u8,

    is_srgb: u8,

    /// 1 when `value_type` is a color, else 0.
    is_color: u8,

    param_key: U32<LE>,

    default_value: [F32<LE>; 5],

    _padding48: Padding<0x08>,
}
