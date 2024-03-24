use std::{
    error::Error,
    future::Future,
    io,
    io::Read,
    pin::{pin, Pin},
    task::Poll,
};

use bevy::{
    app::App,
    asset::{
        io::{AssetSourceId, Reader},
        meta::Settings,
        Asset, AssetLoader, BoxedFuture, LoadContext,
    },
    prelude::{AssetApp, Deref, DerefMut},
};
use futures_lite::{AsyncRead, AsyncReadExt};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};

pub trait FastPathAppExt: AssetApp {
    fn register_fast_path_loader<T: FastPathAssetLoader + 'static>(
        &mut self,
        loader: T,
    ) -> &mut Self;
}

impl FastPathAppExt for App {
    fn register_fast_path_loader<T: FastPathAssetLoader + 'static>(
        &mut self,
        loader: T,
    ) -> &mut Self {
        self.register_asset_loader(FastPathAssetLoaderInstance(loader))
    }
}

pub trait FastPathAssetLoader: Send + Sync {
    type Asset: Asset;

    type Settings: Settings + Default + Serialize + for<'a> Deserialize<'a>;

    /// The type of [error](`std::error::Error`) which could be encountered by this loader.
    type Error: Into<Box<dyn Error + Send + Sync + 'static>> + From<io::Error>;

    fn load_from_bytes<'a>(
        reader: &'a [u8],
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl Future<Output = Result<Self::Asset, Self::Error>> + Send;

    fn extensions(&self) -> &[&str] {
        &[]
    }
}

#[derive(Deref, DerefMut)]
pub struct FastPathAssetLoaderInstance<T: FastPathAssetLoader>(T);

impl<T: FastPathAssetLoader> From<T> for FastPathAssetLoaderInstance<T> {
    fn from(value: T) -> Self {
        FastPathAssetLoaderInstance(value)
    }
}

impl<T: FastPathAssetLoader + Send + Sync + 'static> AssetLoader
    for FastPathAssetLoaderInstance<T>
{
    type Asset = T::Asset;

    type Settings = T::Settings;

    type Error = T::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let dvdbnd_asset_source_id: AssetSourceId = AssetSourceId::from("dvdbnd");
            let vfs_asset_source_id: AssetSourceId = AssetSourceId::from("vfs");

            let source = load_context.asset_path().source();
            let data = if source == &dvdbnd_asset_source_id || source == &vfs_asset_source_id {
                // SAFETY: This invariant is upheld by the `dvdbnd` and `vfs` asset source
                // implementations. They MUST return an implementation of FastPathReader.
                let reader = unsafe { (reader as *mut Reader).cast::<FastPathReader>().as_mut() };
                reader.and_then(|r| r.as_bytes())
            } else {
                None
            };

            match data {
                None => {
                    let mut buffer = Vec::new();
                    reader.read_to_end(&mut buffer).await?;

                    T::load_from_bytes(&buffer, settings, load_context).await
                }
                Some(slice) => T::load_from_bytes(slice, settings, load_context).await,
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        T::extensions(&self.0)
    }
}

/// An [`AsyncRead`] implementation that allows consuming Bevy asset loaders to bypass the read
/// implementation and directly access the data when available.
pub enum FastPathReader<'a> {
    MemoryMapped(Mmap, usize),
    Reader(Box<dyn AsyncRead + Unpin + Send + Sync + 'a>),
    Slice(&'a [u8]),
}

impl<'a> FastPathReader<'a> {
    pub fn as_bytes(&'a self) -> Option<&'a [u8]> {
        match self {
            FastPathReader::Slice(slice) => Some(slice),
            FastPathReader::MemoryMapped(mmap, _) => Some(&mmap[..]),
            FastPathReader::Reader(_) => None,
        }
    }
}

impl<'a> AsyncRead for FastPathReader<'a> {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            FastPathReader::Reader(reader) => AsyncRead::poll_read(pin!(reader), _cx, buf),
            FastPathReader::Slice(slice) => Poll::Ready(Read::read(slice, buf)),
            FastPathReader::MemoryMapped(dvd_bnd, ref mut offset) => {
                let mut data = &dvd_bnd[*offset..];
                let read = match Read::read(&mut data, buf) {
                    Ok(length) => length,
                    Err(e) => return Poll::Ready(Err(e)),
                };

                *offset += read;

                Poll::Ready(Ok(read))
            }
        }
    }
}
