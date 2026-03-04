use core::str;
use std::{borrow::Cow, ffi::CStr, marker::PhantomData};

use thiserror::Error;
use zerocopy::{ByteOrder, FromBytes, FromZeroes, Unaligned, BE, LE, U16, U32, U64};

/// Traits used to represent the varying endiannes, offset size and string encoding
/// used it param files at compile time.
pub mod traits {
    use std::{borrow::Cow, ffi::CStr};

    use utf16string::WStr;
    use zerocopy::{ByteOrder, FromBytes, Unaligned, U16};

    pub trait UnalignedInto<T>: Into<T> + Copy + Unaligned + FromBytes {}
    impl<T, U: Into<T> + Copy + Unaligned + FromBytes> UnalignedInto<T> for U {}

    /// A valid Unicode string slice which may or may not be UTF-8 encoded.
    pub trait GenericStr: ToString {
        /// Returns the length of the string slice in bytes.
        fn len_bytes(&self) -> usize;

        /// Attempt to create a string slice from a null-terminated sequence of bytes.
        fn read_cstr(bytes: &[u8]) -> Option<&'_ Self>;

        /// Convert to a Rust utf-8 string slice, possibly performing a conversion.
        fn to_rust_str(&self) -> Cow<'_, str>;
    }

    impl GenericStr for str {
        fn len_bytes(&self) -> usize {
            self.len()
        }

        fn read_cstr(bytes: &[u8]) -> Option<&'_ Self> {
            let cstr = CStr::from_bytes_until_nul(bytes).ok()?;
            cstr.to_str().ok()
        }

        fn to_rust_str(&self) -> Cow<'_, str> {
            Cow::Borrowed(self)
        }
    }

    impl<BO: ByteOrder> GenericStr for WStr<BO> {
        fn len_bytes(&self) -> usize {
            self.as_bytes().len()
        }

        fn read_cstr(bytes: &[u8]) -> Option<&'_ Self> {
            let nt_pos = bytes.chunks_exact(2).position(|b| b == [0, 0])?;
            WStr::from_utf16(&bytes[..2 * nt_pos]).ok()
        }

        fn to_rust_str(&self) -> Cow<'_, str> {
            Cow::Owned(self.to_string())
        }
    }

    pub trait CharType<BO: ByteOrder> {
        type Unit: UnalignedInto<u32>;
        type Str: GenericStr + ?Sized;
    }

    pub trait OffsetType<BO: ByteOrder> {
        type T: UnalignedInto<u64>;
        type Pad: Unaligned + FromBytes;
    }

    /// Marker type used to represent a param file with 32-bit offsets.
    pub struct Offset32 {}
    impl<BO: ByteOrder> OffsetType<BO> for Offset32 {
        type T = zerocopy::U32<BO>;
        type Pad = [u8; 0];
    }

    /// Marker type used to represent a param file with 64-bit offsets.
    pub struct Offset64 {}
    impl<BO: ByteOrder> OffsetType<BO> for Offset64 {
        type T = zerocopy::U64<BO>;
        type Pad = [u8; 4];
    }

    /// Marker type used to represent a param file with UTF-8 strings.
    pub struct Char {}
    impl<BO: ByteOrder> CharType<BO> for Char {
        type Unit = u8;
        type Str = str;
    }

    /// Marker type used to represent a param file with UTF-16 strings.
    pub struct WChar {}
    impl<BO: ByteOrder> CharType<BO> for WChar {
        type Unit = U16<BO>;
        type Str = WStr<BO>;
    }

    /// Trait containing associated types that vary according to the endianness,
    /// offset size and string encoding of a param file.
    pub trait ParamFileLayout {
        type Endian: ByteOrder;
        type Offset: UnalignedInto<u64>;
        type OffsetPad: Unaligned + FromBytes;
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

use traits::GenericStr;
pub use traits::{Char, CharType, Offset32, Offset64, OffsetType, WChar};

/// Marker type used to encode the possible format traits of a param file at compile time:
/// - Endianness: [`BE`] or [`LE`],
/// - Offset size: [`Offset32`] or [`Offset64`],
/// - String encoding: [`Char`] (single-byte utf-8 strings) or [`WChar`] (wide utf-16 strings).
pub struct ParamFileLayout<E: ByteOrder = LE, O: OffsetType<E> = Offset64, C: CharType<E> = WChar> {
    #[allow(clippy::type_complexity)]
    phantom: PhantomData<fn() -> (E, O, C)>,
}
impl<E: ByteOrder, O: OffsetType<E>, C: CharType<E>> traits::ParamFileLayout
    for ParamFileLayout<E, O, C>
{
    type Endian = E;
    type Offset = O::T;
    type OffsetPad = O::Pad;
    type Char = C::Unit;
    type Str = C::Str;
}

#[repr(C)]
#[derive(Clone, Copy, Unaligned, FromZeroes, FromBytes)]
struct ParamTypeOffset<BO: ByteOrder> {
    unk04: U32<BO>,
    param_type_offset: U64<BO>,
    unk_pad: [u8; 20],
}

#[repr(C)]
#[derive(Clone, Copy, Unaligned, FromZeroes, FromBytes)]
union ParamTypeBlock<BO: ByteOrder> {
    param_type_buf: [u8; 32],
    offset: ParamTypeOffset<BO>,
}

/// Stores information about an individual param row.
#[repr(C)]
#[derive(Clone, Unaligned, FromZeroes, FromBytes)]
pub struct RowDescriptor<T: traits::ParamFileLayout> {
    /// ID of the row. This is unique within the param file.
    pub id: U32<T::Endian>,
    pad: T::OffsetPad,
    /// Offset to the data of the row in the param file.
    pub data_offset: T::Offset,
    /// Offset to the name of the row in the param file if present.
    /// Zero otherwise.
    pub name_offset: T::Offset,
}

impl<T: traits::ParamFileLayout> RowDescriptor<T> {
    /// Gets the data slice for this row.
    ///
    /// If the row size is known for the provided file,
    /// will return the exact slice corresponding to the row data.
    /// Otherwise, the returned slice will go on to the end of the file.
    ///
    /// Returns [`None`] if the resulting slice is out-of-bounds.
    pub fn data<'a>(&self, file: &'a Param<'a, T>) -> Option<&'a [u8]> {
        let offset: usize = self.data_offset.into() as usize;
        match file.detected_row_size {
            Some(size) => file.data.get(offset..offset + size as usize),
            None => file.data.get(offset..),
        }
    }

    /// Gets the name of this row if one is present.
    ///
    /// Returns [`None`] if the name is not present or could not be read due to
    /// out-of-bounds indicies, invalid characters, etc.
    pub fn name<'a>(&self, file: &'a Param<'a, T>) -> Option<&'a T::Str> {
        match self.name_offset.into() as usize {
            0 => None,
            offset => T::Str::read_cstr(file.data.get(offset..)?),
        }
    }
}

#[repr(C)]
#[derive(Clone, Unaligned, FromZeroes, FromBytes)]
pub struct ParamHeader<BO: ByteOrder> {
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
impl<BO: ByteOrder> ParamHeader<BO> {
    /// Total size of this param header in bytes.
    ///
    /// Note that `size_of::<ParamHeader<_>>` is not necessarily equal to this value, as param files
    /// can contain an expanded header populated with 16 bytes of unknown data.
    pub fn header_size(&self) -> usize {
        // From https://github.com/soulsmods/SoulsFormatsNEXT/blob/master/SoulsFormats/Formats/PARAM/PARAM/PARAM.cs#L106
        //
        // The significance of bit 1 of `format_flags_2d` is unknown, but this expression results in
        // the correct header size
        if self.is_64_bit() || (self.has_flag_2d_01() && self.is_32_bit_expanded_header()) {
            0x40
        } else {
            0x30
        }
    }

    /// Whether the param file containing this header uses big-endian encoding.
    pub fn is_big_endian(&self) -> bool {
        self.endianess_flag == 0xFF
    }

    /// Whether the param file containing this header uses UTF-16 encoding for strings,
    /// with endianness of the wide characters determined by [`ParamHeader::is_big_endian`].
    pub fn is_unicode(&self) -> bool {
        (self.format_flags_2e & 1) != 0
    }

    /// If true, this param file uses 32-bit offsets and possibly has an extended header.
    pub fn is_32_bit_expanded_header(&self) -> bool {
        (self.format_flags_2d & 2) != 0
    }

    /// Whether the param file uses 64-bit offsets. If false, it uses 32-bit offsets.
    pub fn is_64_bit(&self) -> bool {
        (self.format_flags_2d & 4) != 0
    }

    /// If true, the param type is stored in the strings section at the end of the param file.
    /// Otherwise, it is stored inline in the header.
    pub fn is_long_param_type(&self) -> bool {
        (self.format_flags_2d & 0x80) != 0
    }

    /// Number of rows in the param file containing this header.
    pub fn row_count(&self) -> usize {
        self.row_count.into()
    }

    pub fn has_flag_2d_01(&self) -> bool {
        (self.format_flags_2d & 1) != 0
    }
}

pub struct Param<'a, T: traits::ParamFileLayout = ParamFileLayout> {
    data: &'a [u8],
    header: &'a ParamHeader<T::Endian>,
    param_type: &'a str,
    row_descriptors: &'a [RowDescriptor<T>],
    detected_strings_offset: Option<u64>,
    detected_row_size: Option<u64>,
    phantom: PhantomData<fn() -> T>,
}

#[derive(Debug, Error)]
pub enum ParamParseError {
    #[error("File does not match param file binary layout")]
    TraitMismatch {
        is_big_endian: bool,
        is_64_bit: bool,
        is_unicode: bool,
    },
    #[error("Invalid data")]
    InvalidData,
}

impl<'a, T: traits::ParamFileLayout> Param<'a, T> {
    /// Attempt to parse the given byte slice as a [`Param`].
    ///
    /// # Errors
    /// Returns a [`ParamParseError`] error if the byte slice contains invalid data,
    /// or if the format traits of the param file (endianness, offset size
    /// and string encoding) don't match with the given [`ParamTraits`].
    ///
    /// To parse a file for which the format traits are not known at compile time,
    /// use [`parse_dyn`].
    ///
    /// # Complexity
    /// This is a zero-copy operation. It usually runs in constant time, but the necessity of
    /// finding the null-terminator to certain strings makes the worst-case linear in the size
    /// of the data.
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
        let row_descriptors = RowDescriptor::<T>::slice_from_prefix(
            data.get(header.header_size()..)
                .ok_or(ParamParseError::InvalidData)?,
            header.row_count(),
        )
        .ok_or(ParamParseError::InvalidData)?
        .0;

        // If we have two rows, try to use them to guess the row size
        if row_descriptors.len() >= 2 {
            let diff = row_descriptors[1]
                .data_offset
                .into()
                .checked_sub(row_descriptors[0].data_offset.into())
                .ok_or(ParamParseError::InvalidData)?;
            detected_row_size = Some(diff);
        }
        if !row_descriptors.is_empty() {
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
            if let Some(offset) = detected_strings_offset {
                let diff = offset
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

    /// Get the header of this param file.
    pub fn header(&self) -> &ParamHeader<T::Endian> {
        self.header
    }

    /// Get the slice of row descriptors of this param file.
    pub fn row_descriptors(&self) -> &[RowDescriptor<T>] {
        self.row_descriptors
    }
}

/// Unified operations on [`Param`] instances that don't depend
/// on the endianness, offset size or string encoding of the underlying file.
///
/// This trait is object safe.
pub trait ParamCommon<'a> {
    /// Returns a byte slice containing the entire param file.
    fn file_bytes(&self) -> &[u8];

    /// Gets the portion of the param file used for storing strings,
    /// provided that the strings offset is known.
    fn strings(&self) -> Option<&[u8]>;

    /// Returns true if the param file is big endian encoded and false otherwise.
    fn is_big_endian(&self) -> bool;

    /// Returns true if the param file uses 64-bit offsets and false otherwise.
    fn is_64_bit(&self) -> bool;

    /// Returns true if the param file stores row names as Unicode (wide) strings and
    /// false otherwise.
    fn is_unicode(&self) -> bool;

    /// Returns the param type (paramdef) of the rows of this param file.
    fn param_type(&self) -> &str;

    /// Returns the number of rows in this param.
    fn row_count(&self) -> usize;

    /// Returns the row size of this param, if known.
    fn row_size(&self) -> Option<usize>;

    /// Checks that the row descriptors of this param are sorted.
    /// If they are not, the following functions will not produce correct results:
    /// - [`ParamCommon::index_of`]
    /// - [`ParamCommon::data_by_id`]
    /// - [`ParamCommon::name_by_id`]
    fn are_rows_sorted(&self) -> bool;

    /// Attempts to find the index of a row given its ID.
    ///
    /// # Constaints
    /// This performs binary search on the row descriptors. As such, if they are not
    /// sorted this will almost certainly lead to bogus results.
    ///
    /// Note that parsing a param file does *NOT* validate that row descriptors are sorted.
    /// You can check this using [`ParamCommon::are_rows_sorted`].
    fn index_of(&self, row_id: u32) -> Option<usize>;

    /// Returns the data for a param row given its index.
    /// To get the data based on the row ID, see [`ParamCommon::data_by_id`].
    ///
    /// May return [`None`] if the file defines an invalid data slice for this row.
    ///
    /// # Panics
    /// Like slice indexing, this function panics if `index` is larger or equal to
    /// [`ParamCommon::row_count`].
    fn data_by_index(&self, index: usize) -> Option<&[u8]>;

    /// Returns the name of a param row given its index, if a name is present.
    /// To get the data based on the row ID, see [`ParamCommon::name_by_id`].
    ///
    /// May return [`None`] if the name is not present or otherwise unreadable.
    ///
    /// # Panics
    /// Like slice indexing, this function panics if `index` is larger or equal to
    /// [`ParamCommon::row_count`].
    fn name_by_index(&self, index: usize) -> Option<Cow<'_, str>>;

    /// Returns the data of a row given its ID.
    ///
    /// # Constaints
    /// This function relies on binary search on the row descriptors. As such, if they are not
    /// sorted this will almost certainly lead to bogus results.
    ///
    /// Note that parsing a param file does *NOT* validate that row descriptors are sorted.
    /// You can check this using [`ParamCommon::are_rows_sorted`].
    fn data_by_id(&self, id: u32) -> Option<&[u8]> {
        self.data_by_index(self.index_of(id)?)
    }

    /// Returns the name of a row given its ID.
    ///
    /// # Constaints
    /// This function relies on binary search on the row descriptors. As such, if they are not
    /// sorted this will almost certainly lead to bogus results.
    ///
    /// Note that parsing a param file does *NOT* validate that row descriptors are sorted.
    /// You can check this using [`ParamCommon::are_rows_sorted`].
    fn name_by_id(&self, id: u32) -> Option<Cow<'_, str>> {
        self.name_by_index(self.index_of(id)?)
    }

    /// Returns a boxed Iterator impl yielding structured information about each param row.
    ///
    /// If working with a concrete type, prefer [`Param::row_descriptors`] to this.
    fn dyn_rows(&self) -> Box<dyn Iterator<Item = UntypedRowInfo<'_>> + '_>;
}

impl<'a, T: traits::ParamFileLayout> ParamCommon<'a> for Param<'a, T> {
    fn file_bytes(&self) -> &[u8] {
        self.data
    }

    fn strings(&self) -> Option<&[u8]> {
        self.detected_strings_offset
            .and_then(|o| self.data.get(o as usize..))
    }

    fn is_big_endian(&self) -> bool {
        T::is_big_endian()
    }

    fn is_unicode(&self) -> bool {
        T::is_unicode()
    }

    fn is_64_bit(&self) -> bool {
        T::is_64_bit()
    }

    fn param_type(&self) -> &str {
        self.param_type
    }

    fn row_count(&self) -> usize {
        self.row_descriptors.len()
    }

    fn row_size(&self) -> Option<usize> {
        self.detected_row_size.map(|r| r as usize)
    }

    fn are_rows_sorted(&self) -> bool {
        self.row_descriptors
            .windows(2)
            .all(|rds| rds[0].id.get() < rds[1].id.get())
    }

    fn data_by_index(&self, index: usize) -> Option<&[u8]> {
        self.row_descriptors[index].data(self)
    }

    fn name_by_index(&self, index: usize) -> Option<Cow<'_, str>> {
        self.row_descriptors[index]
            .name(self)
            .map(|s| s.to_rust_str())
    }

    fn index_of(&self, row_id: u32) -> Option<usize> {
        self.row_descriptors
            .binary_search_by_key(&row_id, |rd| rd.id.into())
            .ok()
    }

    fn dyn_rows(&self) -> Box<dyn Iterator<Item = UntypedRowInfo<'_>> + '_> {
        Box::new(self.row_descriptors.iter().map(|row_desc| UntypedRowInfo {
            id: row_desc.id.get(),
            data_offset: row_desc.data_offset.into(),
            name_offset: row_desc.name_offset.into(),
            data: row_desc.data(self),
            name: row_desc.name(self).map(|s| s.to_rust_str()),
        }))
    }
}

/// Contains untyped param row information.
pub struct UntypedRowInfo<'a> {
    pub id: u32,
    pub data_offset: u64,
    pub name_offset: u64,
    pub data: Option<&'a [u8]>,
    pub name: Option<Cow<'a, str>>,
}

/// Trait to provide an analogue to [`Param::row_descriptors`] of the same name
/// to a [`ParamCommon`] trait object.
pub trait RowDescriptorsDyn {
    /// Returns a boxed Iterator impl yielding structured information about each param row.
    fn row_descriptors(&self) -> Box<dyn Iterator<Item = UntypedRowInfo<'_>> + '_>;
}

impl<'a> RowDescriptorsDyn for dyn ParamCommon<'a> + 'a {
    fn row_descriptors(&self) -> Box<dyn Iterator<Item = UntypedRowInfo<'_>> + '_> {
        self.dyn_rows()
    }
}

/// Parse a param file from a byte slice, returning a boxed [`ParamCommon`] implementation
/// which removes the need from knowing the endianness, offset size and string encoding of
/// the param file at compile time.
///
/// # Errors
/// Returns a [`ParamParseError`] error if the byte slice contains unexpected or invalid data
/// that does not conform to the param file format.
///
/// # Complexity
/// This is a zero-copy operation. It usually runs in constant time, but the necessity of
/// finding the null-terminator to certain strings makes the worst-case linear in the size
/// of the data.
pub fn parse_dyn<'a>(data: &'a [u8]) -> Result<Box<dyn ParamCommon<'a> + 'a>, ParamParseError> {
    let header = ParamHeader::<LE>::ref_from(data).ok_or(ParamParseError::InvalidData)?;
    Ok(
        match (
            header.is_big_endian(),
            header.is_64_bit(),
            header.is_unicode(),
        ) {
            (false, false, false) => {
                Box::new(Param::<ParamFileLayout<LE, Offset32, Char>>::parse(data)?)
            }
            (false, false, true) => {
                Box::new(Param::<ParamFileLayout<LE, Offset32, WChar>>::parse(data)?)
            }
            (false, true, false) => {
                Box::new(Param::<ParamFileLayout<LE, Offset64, Char>>::parse(data)?)
            }
            (false, true, true) => {
                Box::new(Param::<ParamFileLayout<LE, Offset64, WChar>>::parse(data)?)
            }
            (true, false, false) => {
                Box::new(Param::<ParamFileLayout<BE, Offset32, Char>>::parse(data)?)
            }
            (true, false, true) => {
                Box::new(Param::<ParamFileLayout<BE, Offset32, WChar>>::parse(data)?)
            }
            (true, true, false) => {
                Box::new(Param::<ParamFileLayout<BE, Offset64, Char>>::parse(data)?)
            }
            (true, true, true) => {
                Box::new(Param::<ParamFileLayout<BE, Offset64, WChar>>::parse(data)?)
            }
        },
    )
}
