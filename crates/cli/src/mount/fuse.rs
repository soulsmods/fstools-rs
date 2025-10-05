use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    io::{Read, Seek},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, UNIX_EPOCH},
};

use easy_fuser::{
    inode_mapper::{InodeMapper, ValueCreatorParams},
    templates::DefaultFuseHandler,
    types::{
        arguments::{FileAttribute, RequestInfo, SeekFrom},
        errors::{ErrorKind, FuseResult, PosixError},
        file_handle::{BorrowedFileHandle, OwnedFileHandle},
        flags::{FUSEOpenFlags, FUSEOpenResponseFlags, OpenFlags},
        FileKind, Inode,
    },
    FuseHandler,
};
use fstools_dvdbnd::{DvdBnd, DvdBndEntryReader, Name};

const TTL: Duration = Duration::from_secs(300);
const BLOCK_SIZE: u32 = 4096;

#[derive(Debug, Clone)]
enum VfsNodeType {
    Directory,
    File { path: PathBuf, size: u64 },
}

#[derive(Default)]
struct VfsFileDescriptorMap {
    handles: HashMap<u64, DvdBndEntryReader>,
    next_fh: u64,
}

pub struct DvdBndFilesystem {
    dvd_bnd: Arc<DvdBnd>,
    inodes: InodeMapper<VfsNodeType>,
    file_handles: Mutex<VfsFileDescriptorMap>,
    inner: DefaultFuseHandler,
}

impl DvdBndFilesystem {
    pub fn new(dvd_bnd: Arc<DvdBnd>, dictionary: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut inodes = InodeMapper::new(VfsNodeType::Directory);
        let root_inode = inodes.get_root_inode();

        let mut entries = Vec::new();
        for file_path in dictionary {
            let name = Name::from(&file_path);

            if let Some(entry) = dvd_bnd.entry(name) {
                let components: Vec<OsString> = file_path
                    .components()
                    .filter_map(|c| match c {
                        std::path::Component::Normal(s) => Some(s.to_os_string()),
                        _ => None,
                    })
                    .collect();

                let size = entry.file_size();

                entries.push((
                    components,
                    move |_params: ValueCreatorParams<VfsNodeType>| VfsNodeType::File {
                        path: file_path.clone(),
                        size,
                    },
                ));
            }
        }

        // Batch insert all files and directories
        inodes
            .batch_insert(
                &root_inode,
                entries,
                |_params: ValueCreatorParams<VfsNodeType>| VfsNodeType::Directory,
            )
            .expect("Failed to build filesystem tree");

        Self {
            dvd_bnd,
            inodes,
            file_handles: Mutex::new(VfsFileDescriptorMap {
                handles: HashMap::new(),
                next_fh: 1,
            }),
            inner: DefaultFuseHandler::new(),
        }
    }

    fn get_file_attr(&self, inode: &Inode) -> Option<FileAttribute> {
        let info = self.inodes.get(inode)?;

        let (file_type, size) = match info.data {
            VfsNodeType::Directory => (FileKind::Directory, 0),
            VfsNodeType::File { size, .. } => (FileKind::RegularFile, *size),
        };

        Some(FileAttribute {
            size,
            blocks: size.div_ceil(u64::from(BLOCK_SIZE)),
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: file_type,
            perm: if file_type == FileKind::Directory {
                0o555
            } else {
                0o444
            },
            nlink: if file_type == FileKind::Directory {
                2
            } else {
                1
            },
            uid: unsafe { libc::geteuid() },
            gid: unsafe { libc::getegid() },
            rdev: 0,
            blksize: BLOCK_SIZE,
            flags: 0,
            ttl: Some(TTL),
            generation: None,
        })
    }
}
impl FuseHandler<Inode> for DvdBndFilesystem {
    fn get_inner(&self) -> &dyn FuseHandler<Inode> {
        &self.inner
    }

    fn get_default_ttl(&self) -> Duration {
        TTL
    }

    fn lookup(
        &self,
        _req: &RequestInfo,
        parent_id: Inode,
        name: &OsStr,
    ) -> FuseResult<(Inode, FileAttribute)> {
        let result = self
            .inodes
            .lookup(&parent_id, name)
            .ok_or_else(|| PosixError::new(ErrorKind::FileNotFound, String::new()))?;

        let attributes = self
            .get_file_attr(result.inode)
            .ok_or_else(|| PosixError::new(ErrorKind::FileNotFound, String::new()))?;

        Ok((result.inode.clone(), attributes))
    }

    fn getattr(
        &self,
        _req: &RequestInfo,
        file_id: Inode,
        _file_handle: Option<BorrowedFileHandle>,
    ) -> FuseResult<FileAttribute> {
        self.get_file_attr(&file_id)
            .ok_or_else(|| PosixError::new(ErrorKind::FileNotFound, String::new()))
    }

    fn open(
        &self,
        _req: &RequestInfo,
        file_id: Inode,
        _flags: OpenFlags,
    ) -> FuseResult<(OwnedFileHandle, FUSEOpenResponseFlags)> {
        let info = self
            .inodes
            .get(&file_id)
            .ok_or_else(|| PosixError::new(ErrorKind::FileNotFound, String::new()))?;

        match &info.data {
            VfsNodeType::Directory => Err(PosixError::new(ErrorKind::IsADirectory, String::new())),
            VfsNodeType::File { path, .. } => {
                let reader = self.dvd_bnd.open(path).map_err(|_| {
                    PosixError::new(ErrorKind::InputOutputError, path.clone().to_string_lossy())
                })?;

                let mut handles = self
                    .file_handles
                    .lock()
                    .expect("file handle mutex poisoned");
                let fh = handles.next_fh;
                handles.next_fh += 1;
                handles.handles.insert(fh, reader);

                let owned = unsafe { OwnedFileHandle::from_raw(fh) };
                Ok((owned, FUSEOpenResponseFlags::empty()))
            }
        }
    }

    fn read(
        &self,
        _req: &RequestInfo,
        _file_id: Inode,
        file_handle: BorrowedFileHandle,
        seek: SeekFrom,
        size: u32,
        _flags: FUSEOpenFlags,
        _lock_owner: Option<u64>,
    ) -> FuseResult<Vec<u8>> {
        let mut handles = self
            .file_handles
            .lock()
            .expect("file handle mutex poisoned");

        let reader = handles
            .handles
            .get_mut(&file_handle.as_raw())
            .ok_or_else(|| PosixError::new(ErrorKind::BadFileDescriptor, String::new()))?;

        let seek_from = match seek {
            SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset),
            SeekFrom::End(offset) => std::io::SeekFrom::End(offset),
            SeekFrom::Current(offset) => std::io::SeekFrom::Current(offset),
        };

        reader
            .seek(seek_from)
            .map_err(|err| PosixError::new(ErrorKind::InputOutputError, err.to_string()))?;

        let mut buffer = vec![0u8; size as usize];
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|err| PosixError::new(ErrorKind::InputOutputError, err.to_string()))?;
        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    fn readdir(
        &self,
        _req: &RequestInfo,
        file_id: Inode,
        _file_handle: BorrowedFileHandle,
    ) -> FuseResult<Vec<(OsString, (Inode, FileKind))>> {
        let info = self
            .inodes
            .get(&file_id)
            .ok_or_else(|| PosixError::new(ErrorKind::FileNotFound, String::new()))?;

        if !matches!(info.data, VfsNodeType::Directory) {
            return Err(PosixError::new(
                ErrorKind::NotADirectory,
                "requested inode is not a directory".to_string(),
            ));
        }

        let mut entries = Vec::new();
        entries.push((OsString::from("."), (file_id.clone(), FileKind::Directory)));

        let parent_inode = info.parent.clone();
        entries.push((
            OsString::from(".."),
            (parent_inode.clone(), FileKind::Directory),
        ));

        for (name, child_inode) in self.inodes.get_children(&file_id) {
            let child_info = self
                .inodes
                .get(child_inode)
                .ok_or_else(|| PosixError::new(ErrorKind::FileNotFound, String::new()))?;

            let kind = match child_info.data {
                VfsNodeType::Directory => FileKind::Directory,
                VfsNodeType::File { .. } => FileKind::RegularFile,
            };

            entries.push((name.as_os_str().to_os_string(), (child_inode.clone(), kind)));
        }

        Ok(entries)
    }

    fn release(
        &self,
        _req: &RequestInfo,
        _file_id: Inode,
        file_handle: OwnedFileHandle,
        _flags: OpenFlags,
        _lock_owner: Option<u64>,
        _flush: bool,
    ) -> FuseResult<()> {
        let mut handles = self
            .file_handles
            .lock()
            .expect("file handle mutex poisoned");
        handles.handles.remove(&file_handle.as_raw());
        Ok(())
    }
}
