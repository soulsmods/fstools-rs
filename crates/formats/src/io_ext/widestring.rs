use thiserror::Error;
use utf16string::WStr;
use zerocopy::ByteOrder;

#[derive(Debug, Error)]
pub enum ReadWidestringError {
    #[error("Could not find end of string")]
    NoEndFound,

    #[error("String is not valid UTF-16")]
    InvalidUTF16,
}

/// Reads a null-terminated widestring from an input slice.
/// Fails if the null terminator cannot be found or if the input is invalid UTF-16.
pub fn read_wide_cstring<BO: ByteOrder>(input: &[u8]) -> Result<&WStr<BO>, ReadWidestringError> {
    // Find the end of the input string. Unfortunately we need the end to be
    // known as U16Str seems to behave inconsistently (sometimes yields garble
    // at the end of a read string) when the slice doesn't end at the terminator.
    let length = input
        .chunks_exact(2)
        .position(|bytes| bytes == [0, 0])
        .ok_or(ReadWidestringError::NoEndFound)?;

    // Create a view that has a proper end
    let string_bytes = &input[..length * 2];
    WStr::from_utf16(string_bytes).map_err(|_e| ReadWidestringError::InvalidUTF16)
}
