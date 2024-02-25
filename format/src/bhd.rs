use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    mem::{transmute, MaybeUninit},
};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use rayon::{iter::ParallelIterator, prelude::*};
use rsa::{pkcs1::DecodeRsaPublicKey, traits::PublicKeyParts, RsaPublicKey};
use rug::{integer::Order, Integer};

use crate::io_ext::ReadFormatsExt;

pub struct BhdKey {
    exponent: Integer,
    modulus: Integer,
    size: usize,
}

pub type BhdKeyDecodeError = rsa::Error;

impl BhdKey {
    pub fn from_pem(data: &str) -> Result<Self, BhdKeyDecodeError> {
        let key = RsaPublicKey::from_pkcs1_pem(data)?;
        let exponent = Integer::from_digits(&key.e().to_bytes_be(), Order::Msf);
        let modulus = Integer::from_digits(&key.n().to_bytes_be(), Order::Msf);
        let size = (modulus.significant_bits() as usize + 7) / 8;

        Ok(BhdKey {
            exponent,
            modulus,
            size,
        })
    }
}

pub struct Bhd {
    pub toc: Vec<BhdTocEntry>,
}

#[derive(Debug)]
pub struct BhdTocEntry {
    pub hash: u64,
    pub padded_size: u32,
    pub size: u32,
    pub offset: u64,
    pub aes_key: [u8; 16],
    pub encrypted_ranges: Vec<(i64, i64)>,
}

#[derive(Debug)]
pub struct BhdHeader {
    pub is_big_endian: bool,
    pub file_size: u32,
    pub buckets: u32,
    pub buckets_offset: u32,
    pub salt_length: u32,
    pub salt: Vec<u8>,
}

impl Bhd {
    pub fn read<R: Read + Seek>(mut file: R, key: BhdKey) -> Result<Self, std::io::Error> {
        let key_size = key.size;
        let file_len = file.seek(SeekFrom::End(0))? as usize;
        let decrypted_file_len = file_len - file_len / key_size;
        file.seek(SeekFrom::Start(0))?;

        let mut decrypted_data = vec![MaybeUninit::uninit(); decrypted_file_len];
        let mut encrypted_data = Vec::with_capacity(file_len);
        file.read_to_end(&mut encrypted_data)?;

        let decrypted_len = encrypted_data
            .par_chunks(key_size)
            .zip(decrypted_data.par_chunks_mut(key_size - 1))
            .map(|(encrypted_block, decrypted_block)| {
                let mut decrypted = Integer::from_digits(encrypted_block, Order::Msf);
                decrypted
                    .pow_mod_mut(&key.exponent, &key.modulus)
                    .expect("failed to decrypt");

                let mut decrypted_with_padding = vec![MaybeUninit::<u8>::uninit(); key_size];
                decrypted.write_digits(
                    unsafe { transmute::<_, &mut [u8]>(&mut decrypted_with_padding[..]) },
                    Order::Msf,
                );
                decrypted_block.copy_from_slice(&decrypted_with_padding[1..]);

                Ok::<_, std::io::Error>(key_size)
            })
            .try_reduce(|| 0, |len, block_len| Ok(len + block_len))?;

        // SAFETY: all elements from [0,decrypted_len) have been initialized.
        let decrypted_data: Vec<u8> = unsafe {
            decrypted_data.set_len(decrypted_len);
            transmute(decrypted_data)
        };

        let mut reader = Cursor::new(&decrypted_data[..]);
        let header = read_header(&mut reader)?;

        let toc = if header.is_big_endian {
            read_toc::<_, BigEndian>(header.buckets as usize, reader)
        } else {
            read_toc::<_, LittleEndian>(header.buckets as usize, reader)
        }?;

        Ok(Bhd { toc })
    }
}

pub fn read_header_data<R: Read, O: ByteOrder>(
    mut reader: R,
    is_big_endian: bool,
) -> Result<BhdHeader, std::io::Error> {
    reader.read_padding(7)?;

    let file_size = reader.read_u32::<O>()?;
    let toc_buckets = reader.read_i32::<O>()?;
    let toc_offset = reader.read_i32::<O>()?;
    let salt_length = reader.read_u32::<O>()?;

    let mut salt = vec![0u8; salt_length as usize];
    reader.read_exact(&mut salt)?;

    Ok(BhdHeader {
        is_big_endian,
        file_size,
        buckets: toc_buckets as u32,
        buckets_offset: toc_offset as u32,
        salt_length,
        salt,
    })
}

pub fn read_header<R: Read>(mut reader: R) -> Result<BhdHeader, std::io::Error> {
    reader.read_magic(b"BHD5")?;

    let endianness = reader.read_i8()?;
    if endianness == -1 {
        read_header_data::<_, LittleEndian>(reader, false)
    } else {
        read_header_data::<_, BigEndian>(reader, true)
    }
}

pub fn read_toc<R: Read + Seek, O: ByteOrder>(
    buckets: usize,
    mut reader: R,
) -> Result<Vec<BhdTocEntry>, std::io::Error> {
    let mut entries = Vec::new();

    // TODO: split some of this out
    for _ in 0..buckets {
        let entry_count = reader.read_u32::<O>()?;
        let entry_data_offset = reader.read_u32::<O>()?;

        let next_bucket_pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(entry_data_offset as u64))?;

        for _ in 0..entry_count {
            let hash = reader.read_u64::<O>()?;
            let padded_size = reader.read_u32::<O>()?;
            let size = reader.read_u32::<O>()?;
            let offset = reader.read_u64::<O>()?;

            let _digest_offset = reader.read_u64::<O>()?;
            let encryption_offset = reader.read_u64::<O>()?;

            let next_file_pos = reader.stream_position()?;
            let mut aes_key = [0u8; 16];

            let mut encrypted_ranges = Vec::new();

            if encryption_offset != 0 {
                reader.seek(SeekFrom::Start(encryption_offset))?;

                reader.read_exact(&mut aes_key)?;

                let encrypted_range_count = reader.read_u32::<O>()?;

                for _ in 0..encrypted_range_count {
                    encrypted_ranges.push((reader.read_i64::<O>()?, reader.read_i64::<O>()?));
                }
            }

            reader.seek(SeekFrom::Start(next_file_pos))?;

            entries.push(BhdTocEntry {
                hash,
                padded_size,
                size,
                offset,
                aes_key,
                encrypted_ranges,
            })
        }

        reader.seek(SeekFrom::Start(next_bucket_pos))?;
    }

    Ok(entries)
}
