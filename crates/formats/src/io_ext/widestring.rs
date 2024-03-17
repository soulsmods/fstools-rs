use std::borrow::Cow;
use thiserror::Error;
use bytemuck::PodCastError;
use widestring::{U16Str, U16String};

#[derive(Debug, Error)]
pub enum ReadWidestringError {
    #[error("Could not find end of string")]
    NoEndFound,

    #[error("Bytemuck could not cast input slice to a u16 slice")]
    Cast,
}

/// Reads a widestring from an input slice.
/// Attempts to read a widestring in-place and copies the string bytes 
/// to aligned memory when that fails.
pub fn read_widestring(input: &[u8]) -> Result<Cow<'_, U16Str>, ReadWidestringError> {
    // Find the end of the input string. Unfortunately we need the end to be 
    // known as U16Str seems to behave inconsistently (sometimes yields garble
    // at the end of a read string) when the slice doesn't end at the terminator.
    let length = input
        .chunks_exact(2)
        .position(|bytes| bytes[0] == 0x0 && bytes[1] == 0x0)
        .ok_or(ReadWidestringError::NoEndFound)?;

    // Create a view that has a proper end so we don't copy
    // the entire input slice if required and so we don't have to deal with
    // bytemuck freaking out over the end not being aligned
    let string_bytes = &input[..length * 2];

    Ok(match bytemuck::try_cast_slice::<u8, u16>(string_bytes) {
        Ok(s) => Cow::Borrowed(U16Str::from_slice(s)),
        Err(e) => {
            // We should probably return the error if it isn't strictly
            // about the alignment of the input.
            if e != PodCastError::TargetAlignmentGreaterAndInputNotAligned {
                return Err(ReadWidestringError::Cast);
            }

            let aligned_copy = string_bytes
                .chunks(2)
                .map(|a| u16::from_le_bytes([a[0], a[1]]))
                .collect::<Vec<u16>>();

            Cow::Owned(U16String::from_vec(aligned_copy))
        }
    })
}
