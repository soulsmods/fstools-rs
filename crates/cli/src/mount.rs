use std::{path::Path, sync::Arc};

use color_eyre::eyre::{Result, WrapErr};
use fstools_dvdbnd::DvdBnd;

#[cfg(target_os = "linux")]
mod fuse;

#[cfg(target_os = "linux")]
pub fn mount_filesystem(dvd_bnd: Arc<DvdBnd>, mount_point: &Path) -> Result<()> {
    use easy_fuser::prelude::MountOption;

    let filesystem = fuse::DvdBndFilesystem::new(dvd_bnd, fstools_elden_ring_support::dictionary());
    let options = vec![MountOption::RO, MountOption::FSName("fstools".to_string())];

    println!("Mounting filesystem at {}", mount_point.display());
    println!("Use 'fusermount -u {}' to unmount", mount_point.display());

    easy_fuser::mount(filesystem, mount_point, &options, num_cpus::get())
        .with_context(|| format!("failed to mount filesystem at {}", mount_point.display()))?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn mount_filesystem(_dvd_bnd: Arc<DvdBnd>, _mount_point: &Path) -> Result<()> {
    Err(color_eyre::eyre::eyre!(
        "mounting is not supported on this platform"
    ))
}
