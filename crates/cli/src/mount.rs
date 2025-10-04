use std::{path::Path, sync::Arc};

use fstools_dvdbnd::DvdBnd;

#[cfg(target_os = "linux")]
mod fuse;

#[cfg(target_os = "linux")]
pub fn mount_filesystem(
    dvd_bnd: Arc<DvdBnd>,
    mount_point: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use easy_fuser::prelude::MountOption;

    let filesystem = fuse::DvdBndFilesystem::new(dvd_bnd, fstools_elden_ring_support::dictionary());
    let options = vec![MountOption::RO, MountOption::FSName("fstools".to_string())];

    println!("Mounting filesystem at {}", mount_point.display());
    println!("Use 'fusermount -u {}' to unmount", mount_point.display());

    easy_fuser::mount(filesystem, mount_point, &options)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn mount_filesystem(
    dvd_bnd: Arc<DvdBnd>,
    game_type: GameType,
    mount_point: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("unsupported")
}
