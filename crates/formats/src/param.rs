use core::str;
use std::{ffi::CStr, marker::PhantomData};

use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, Unaligned, LE, U16, U32, U64};

pub mod traits {
    use std::{borrow::Cow, ffi::CStr};
    use utf16string::WStr;
    use zerocopy::{ByteOrder, FromBytes, Unaligned, U16};

    pub trait StaticBO: 'static + ByteOrder {}
    impl<BO: 'static + ByteOrder> StaticBO for BO {}

    pub trait UnalignedInto<T>: Into<T> + Copy + Unaligned + FromBytes {}
    impl<T, U: Into<T> + Copy + Unaligned + FromBytes> UnalignedInto<T> for U {}

    pub trait GenericStr: ToString {
        /// Returns the length of a string of [`Char`] in bytes.
        fn len_bytes(&self) -> usize;

        /// Attempt to read a null-terminated string of [`Char`].
        fn read_nt_str(bytes: &[u8]) -> Option<&'_ Self>;

        /// Convert to a Rust string, possibly performing a conversion.
        fn to_rust_str(&self) -> Cow<'_, str>;
    }

    impl GenericStr for str {
        fn len_bytes(&self) -> usize {
            self.as_bytes().len()
        }

        fn read_nt_str(bytes: &[u8]) -> Option<&'_ Self> {
            let cstr = CStr::from_bytes_until_nul(bytes).ok()?;
            cstr.to_str().ok()
        }

        fn to_rust_str(&self) -> Cow<'_, str> {
            Cow::Borrowed(self)
        }
    }

    impl<BO: StaticBO> GenericStr for WStr<BO> {
        fn len_bytes(&self) -> usize {
            self.as_bytes().len()
        }

        fn read_nt_str(bytes: &[u8]) -> Option<&'_ Self> {
            let nt_pos = bytes.chunks_exact(2).position(|b| b == &[0, 0])?;
            WStr::from_utf16(&bytes[..2 * nt_pos]).ok()
        }

        fn to_rust_str(&self) -> Cow<'_, str> {
            Cow::Owned(self.to_string())
        }
    }

    pub trait CharType<BO: StaticBO> {
        type Unit: UnalignedInto<u32>;
        type Str: GenericStr + ?Sized;
    }

    pub trait OffsetType<BO: StaticBO> {
        type T: UnalignedInto<u64>;
    }

    pub struct Offset32 {}
    impl<BO: StaticBO> OffsetType<BO> for Offset32 {
        type T = zerocopy::U32<BO>;
    }

    pub struct Offset64 {}
    impl<BO: StaticBO> OffsetType<BO> for Offset64 {
        type T = zerocopy::U64<BO>;
    }

    pub struct Char {}
    impl<BO: StaticBO> CharType<BO> for Char {
        type Unit = u8;
        type Str = str;
    }

    pub struct WChar {}
    impl<BO: StaticBO> CharType<BO> for WChar {
        type Unit = U16<BO>;
        type Str = WStr<BO>;
    }

    pub trait ParamTraits {
        type Endian: StaticBO;
        type Offset: UnalignedInto<u64>;
        type Char: UnalignedInto<u32>;
        type Str: GenericStr + ?Sized;

        fn is_unicode() -> bool {
            std::mem::size_of::<Self::Char>() == 2
        }

        fn is_64_bit() -> bool {
            std::mem::size_of::<Self::Offset>() == 8
        }

        fn is_big_endian() -> bool {
            Self::Endian::read_u16(&[0, 1]) == 1
        }
    }
}

pub use traits::{Char, CharType, Offset32, Offset64, OffsetType, StaticBO, WChar};

pub struct ParamTraits<E: StaticBO = LE, O: OffsetType<E> = Offset64, C: CharType<E> = WChar> {
    phantom: PhantomData<fn() -> (E, O, C)>,
}
impl<E: StaticBO, O: OffsetType<E>, C: CharType<E>> traits::ParamTraits for ParamTraits<E, O, C> {
    type Endian = E;
    type Offset = O::T;
    type Char = C::Unit;
    type Str = C::Str;
}

#[repr(C)]
#[derive(Clone, Copy, Unaligned, FromZeroes, FromBytes)]
struct ParamTypeOffset<BO: StaticBO> {
    unk04: U32<BO>,
    param_type_offset: U64<BO>,
    unk_pad: [u8; 20],
}

#[repr(C)]
#[derive(Clone, Copy, Unaligned, FromZeroes, FromBytes)]
union ParamTypeBlock<BO: StaticBO> {
    param_type_buf: [u8; 32],
    offset: ParamTypeOffset<BO>,
}

#[repr(C)]
#[derive(Clone, Unaligned, FromZeroes, FromBytes)]
pub struct RowDescriptor<I> {
    pub id: I,
    pub data_offset: I,
    pub name_offset: I,
}

#[repr(C)]
#[derive(Clone, Unaligned, FromZeroes, FromBytes)]
pub struct ParamHeader<BO: StaticBO> {
    strings_offset: U32<BO>,
    short_data_offset: U16<BO>,
    unk006: U16<BO>,
    paramdef_data_version: U16<BO>,
    row_count: U16<BO>,
    param_type_block: ParamTypeBlock<BO>,
    endianess_flag: u8,
    format_flags_2d: u8,
    format_flags_2e: u8,
    paramdef_format_version: u8,
}
impl<BO: StaticBO> ParamHeader<BO> {
    pub fn header_size(&self) -> usize {
        let f = self.format_flags_2d;
        if (f & 3) == 3 || (f & 4) != 0 {
            0x40
        } else {
            0x30
        }
    }

    pub fn is_big_endian(&self) -> bool {
        self.endianess_flag == 0xFF
    }

    pub fn is_unicode(&self) -> bool {
        return (self.format_flags_2e & 1) != 0;
    }

    pub fn is_64_bit(&self) -> bool {
        (self.format_flags_2d & 4) != 0
    }

    pub fn is_long_param_type(&self) -> bool {
        (self.format_flags_2d & 0x80) != 0
    }

    pub fn row_count(&self) -> usize {
        self.row_count.into()
    }
}

pub struct Param<'a, T: traits::ParamTraits = ParamTraits> {
    data: &'a [u8],
    header: &'a ParamHeader<T::Endian>,
    param_type: &'a str,
    row_descriptors: &'a [RowDescriptor<T::Offset>],
    detected_strings_offset: Option<u64>,
    detected_row_size: Option<u64>,
    phantom: PhantomData<fn() -> T>,
}

#[derive(Debug, Error)]
pub enum ParamParseError {
    #[error("File does not match param file traits")]
    TraitMismatch {
        is_big_endian: bool,
        is_64_bit: bool,
        is_unicode: bool,
    },
    #[error("Invalid data")]
    InvalidData,
}

impl<'a, T: traits::ParamTraits> Param<'a, T> {
    pub fn parse(data: &'a [u8]) -> Result<Self, ParamParseError> {
        let header = ParamHeader::ref_from(data).ok_or(ParamParseError::InvalidData)?;

        // Check if this param file is compatible with our traits
        if T::is_64_bit() != header.is_64_bit()
            || T::is_unicode() != header.is_unicode()
            || T::is_big_endian() != header.is_big_endian()
        {
            return Err(ParamParseError::TraitMismatch {
                is_big_endian: header.is_big_endian(),
                is_64_bit: header.is_64_bit(),
                is_unicode: header.is_unicode(),
            });
        }

        let mut detected_strings_offset: Option<u64> = None;
        let mut detected_row_size: Option<u64> = None;

        // Parse the param type string (expecting ASCII, so failing if not utf8 is fine)
        let param_type = if header.is_long_param_type() {
            // SAFETY: union access is always safe for FromBytes
            let inlined = unsafe { &header.param_type_block.param_type_buf };
            let inlined_nt = &inlined[..inlined
                .iter()
                .position(|b| *b == 0)
                .unwrap_or(inlined.len())];

            str::from_utf8(inlined_nt).map_err(|_| ParamParseError::InvalidData)?
        } else {
            // SAFETY: union access is always safe for FromBytes
            let offset: u64 = unsafe { header.param_type_block.offset.param_type_offset }.into();
            let strings = data
                .get(offset as usize..)
                .ok_or(ParamParseError::InvalidData)?;

            // This is going to be the best guess for the strings offset
            detected_strings_offset = Some(offset);

            CStr::from_bytes_until_nul(strings)
                .ok()
                .and_then(|cstr| cstr.to_str().ok())
                .ok_or(ParamParseError::InvalidData)?
        };

        // Parse the row descriptors
        let row_descriptors = RowDescriptor::<T::Offset>::slice_from_prefix(
            data.get(header.header_size()..)
                .ok_or(ParamParseError::InvalidData)?,
            header.row_count(),
        )
        .ok_or(ParamParseError::InvalidData)?
        .0;

        // If we have two rows, try to use them to guess the row size
        if row_descriptors.len() >= 2 {
            let diff = (&row_descriptors[1])
                .data_offset
                .into()
                .checked_sub(row_descriptors[0].data_offset.into())
                .ok_or(ParamParseError::InvalidData)?;
            detected_row_size = Some(diff);
        }
        if row_descriptors.len() >= 1 {
            // If the row name offset is set, this will be the string offset
            // provided it is not already set (as we have one row).
            // Otherwise, fall back to the (very unreliable) field in the header.
            let row_0_name_ofs = row_descriptors[0].name_offset.into();
            if row_0_name_ofs != 0 {
                detected_strings_offset.get_or_insert(row_0_name_ofs);
            } else if header.strings_offset != U32::ZERO {
                detected_strings_offset.get_or_insert(header.strings_offset.into());
            }
            // If we have a single row and the strings offset is known,
            // then the row size will be the difference between the row data and strings
            if let Some(o) = detected_strings_offset {
                let diff = o
                    .checked_sub(row_descriptors[0].data_offset.into())
                    .ok_or(ParamParseError::InvalidData)?;
                detected_row_size.get_or_insert(diff);
            }
        }

        Ok(Self {
            data,
            header,
            param_type,
            row_descriptors,
            detected_row_size,
            detected_strings_offset,
            phantom: PhantomData,
        })
    }
}
