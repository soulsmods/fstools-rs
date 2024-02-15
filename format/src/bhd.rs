use std::io::{self, Read, Seek, SeekFrom};
use aes::{cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit}, Aes128};
use byteorder::{ReadBytesExt, LE};
use openssl::{error::ErrorStack, rsa::{Padding, Rsa}};

const KEY_SIZE: usize = 256;

type BHDReader = std::io::Cursor<Vec<u8>>;

#[derive(Debug)]
pub struct BHD {
    pub endianness: i8,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub unk4: u32,
    pub file_size: u32,
    pub bucket_count: u32,
    pub bucket_offset: u32,
    pub salt_length: u32,
    pub salt: Vec<u8>,
    pub buckets: Vec<Bucket>,
}

#[derive(Debug)]
pub enum BHDError {
    IO(io::Error),
    Rsa(ErrorStack),
}

impl BHD {
    pub fn from_reader_with_key(
        r: &mut std::fs::File,
        key: &[u8]
    ) -> Result<Self, BHDError> {
        let mut buffer = {
            let mut b = Vec::new();
            r.read_to_end(&mut b)
                .map_err(BHDError::IO)?;
            b
        };

        let public_key = Rsa::public_key_from_pem_pkcs1(key)
            .map_err(BHDError::Rsa)?;

        assert!(
            public_key.size() as usize == KEY_SIZE,
            "Wrong key size",
        );

        // Loop over the encrypted data and decrypt in-place
        let mut decrypt_offset = 0;
        let mut target_offset = 0;
        while decrypt_offset < buffer.len() {
            let mut decrypt_buffer: [u8; KEY_SIZE] = [0x0u8; KEY_SIZE];

            // Grab a chunk of KEY_SIZE to decrypt
            let chunk = &mut buffer[decrypt_offset..decrypt_offset+KEY_SIZE];

            // Decrypt into temp buffer
            let decrypted_length = public_key.public_decrypt(
                chunk,
                &mut decrypt_buffer,
                Padding::NONE
            ).map_err(BHDError::Rsa)?;

            // Overwrite the original bytes with the decrypted ones
            buffer[target_offset..(target_offset + KEY_SIZE) - 1]
                .copy_from_slice(&decrypt_buffer[1..]);

            // Move offsets
            decrypt_offset += decrypted_length;
            target_offset += decrypted_length - 1;
        }

        // Truncate the final buffer to the amount of decrypted bytes
        buffer.truncate(target_offset);

        // Move buffer into cursor
        let mut decrypted_reader = std::io::Cursor::new(buffer);

        Ok(Self::from_reader(&mut decrypted_reader).map_err(BHDError::IO)?)
    }

    fn from_reader(r: &mut BHDReader) -> Result<Self, io::Error> {
        assert!(
            r.read_u32::<LE>()? == 0x35444842,
            "Magic was not correct",
        );

        let endianness = r.read_i8()?;
        let unk1 = r.read_u8()?;
        let unk2 = r.read_u8()?;
        let unk3 = r.read_u8()?;
        let unk4 = r.read_u32::<LE>()?;
        let file_size = r.read_u32::<LE>()?;
        let bucket_count = r.read_u32::<LE>()?;
        let bucket_offset = r.read_u32::<LE>()?;
        let salt_length = r.read_u32::<LE>()?;

        let mut salt = vec![0u8; salt_length as usize];
        r.read_exact(salt.as_mut_slice())?;

        r.seek(SeekFrom::Start(bucket_offset as u64))?;

        let mut buckets = vec![];
        for _ in 0..bucket_count {
            buckets.push(Bucket::from_reader(r)?);
        }

        Ok(Self {
            endianness,
            unk1,
            unk2,
            unk3,
            unk4,
            file_size,
            bucket_count,
            bucket_offset,
            salt_length,
            salt,
            buckets,
        })
    }
}

#[derive(Debug)]
pub struct Bucket {
    pub files: Vec<FileDescriptor>,
}

impl Bucket {
    pub fn from_reader(r: &mut BHDReader) -> Result<Self, io::Error> {
        let count = r.read_u32::<LE>()?;
        let offset = r.read_u32::<LE>()?;

        let current = r.seek(SeekFrom::Current(0))?;
        r.seek(SeekFrom::Start(offset as u64))?;

        let mut files = vec![];
        for _ in 0..count {
            files.push(FileDescriptor::from_reader(r)?);
        }

        r.seek(SeekFrom::Start(current))?;

        Ok(Self { files })
    }
}

#[derive(Debug)]
pub struct FileDescriptor {
    pub file_path_hash: u64,
    pub padded_file_size: u32,
    pub file_size: u32,
    pub file_offset: u64,
    pub sha_offset: u64,
    pub aes_key: [u8; 16],
    pub aes_ranges: Vec<(i64, i64)>,
}

impl FileDescriptor {
    pub fn from_reader(r: &mut BHDReader) -> Result<Self, io::Error> {
        let file_path_hash = r.read_u64::<LE>()?;
        let padded_file_size = r.read_u32::<LE>()?;
        let file_size = r.read_u32::<LE>()?;
        let file_offset = r.read_u64::<LE>()?;
        let sha_offset = r.read_u64::<LE>()?;
        let aes_key_offset = r.read_u64::<LE>()?;

        let mut aes_ranges = Vec::new();
        let current_position = r.seek(io::SeekFrom::Current(0))?;
        r.seek(io::SeekFrom::Start(aes_key_offset))?;

        let mut aes_key = [0u8; 16];
        r.read_exact(&mut aes_key)?;

        let aes_range_count = r.read_u32::<LE>()?;
        for _ in 0..aes_range_count {
            aes_ranges.push((r.read_i64::<LE>()?, r.read_i64::<LE>()?));
        }

        r.seek(io::SeekFrom::Start(current_position))?;

        Ok(Self {
            file_path_hash,
            padded_file_size,
            file_size,
            file_offset,
            sha_offset,
            aes_key,
            aes_ranges,
        })
    }

    pub fn decrypt_file(&self, data: &mut [u8]) {
        let key = GenericArray::from_slice(&self.aes_key);
        let cipher = Aes128::new(&key);

        for (start, end) in self.aes_ranges.iter() {
            if *start == -1 {
                continue;
            }

            let encrypted_range = &mut data[*start as usize..*end as usize];

            // Decrypt by chunks of key size.
            encrypted_range.chunks_mut(16)
                .map(|c| GenericArray::from_mut_slice(c))
                .for_each(|b| cipher.decrypt_block(b));
        }
    }
}

use std::num::Wrapping;

const HASH_PRIME: Wrapping<u64> = Wrapping(0x85);

pub fn hash_path(input: &str) -> u64 {
    let mut result = Wrapping(0);

    for character in input.as_bytes() {
        result = result * HASH_PRIME + Wrapping(*character as u64);
    }

    result.0
}
