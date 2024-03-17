use bevy::{
    asset::{Assets, Handle},
    prelude::{AssetServer, Deref, DerefMut, Res, ResMut, Resource},
    tasks::IoTaskPool,
};
use fstools_asset_server::{
    asset_source::vfs::Vfs,
    types::bnd4::{Archive, ArchiveEntry},
};

#[derive(Default, Deref, DerefMut, Resource)]
pub struct ArchivesLoading(pub(crate) Vec<Handle<Archive>>);

pub fn vfs_mount_system(
    mut archives_loading: ResMut<ArchivesLoading>,
    mut archives: ResMut<Assets<Archive>>,
    mut archive_entries: ResMut<Assets<ArchiveEntry>>,
    asset_server: Res<AssetServer>,
    vfs: ResMut<Vfs>,
) {
    let mut still_loading = vec![];

    for archive_handle in archives_loading.drain(..) {
        if !asset_server.is_loaded_with_dependencies(&archive_handle) {
            still_loading.push(archive_handle);
            continue;
        }

        let archive = archives.remove(archive_handle).expect("bad archive id");
        let entries: Vec<_> = archive
            .files
            .iter()
            .map(|(name, handle)| {
                (
                    name.clone(),
                    archive_entries.remove(handle).expect("no data").data,
                )
            })
            .collect();

        let io_pool = IoTaskPool::get();
        let mut vfs = vfs.clone();

        io_pool
            .spawn(async move {
                for (name, data) in entries {
                    vfs.mount_file(name, data);
                }
            })
            .detach();
    }

    archives_loading.extend(still_loading);
}
