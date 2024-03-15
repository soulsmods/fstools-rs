use bevy::{
    asset::{AssetEvent, Assets, Handle},
    log::info,
    prelude::{AssetServer, EventReader, NextState, Res, ResMut, Resource, States},
    tasks::IoTaskPool,
};
use fstools_asset_server::{
    types::bnd4::{Archive, ArchiveEntry},
    vfs::Vfs,
};
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreloadingState {
    #[default]
    Preloading,
    Loaded,
}

#[derive(Default, Resource)]
pub struct ArchivesLoading(pub(crate) Vec<Handle<Archive>>);

pub fn vfs_mount_system(
    mut archives_loading: ResMut<ArchivesLoading>,
    archives: Res<Assets<Archive>>,
    archive_entries: Res<Assets<ArchiveEntry>>,
    asset_server: Res<AssetServer>,
    vfs: ResMut<Vfs>,
) {
    let mut still_loading = vec![];

    for archive in archives_loading.0.drain(..) {
        if !asset_server.is_loaded_with_dependencies(&archive) {
            still_loading.push(archive);
            continue;
        }

        let archive = archives.get(archive).expect("bad archive id");

        for (name, entry) in archive.files.iter() {
            let io_pool = IoTaskPool::get();

            let name = name.clone();
            let data = archive_entries.get(entry).expect("no data").data.clone();
            let mut vfs = vfs.clone();

            io_pool
                .spawn(async move {
                    vfs.mount_file(name, data);
                })
                .detach();
        }
    }

    archives_loading.0.extend(still_loading);
}
