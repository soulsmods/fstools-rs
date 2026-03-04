mod reader;
mod state;

use std::sync::{atomic::Ordering, Arc};

use bevy::{
    asset::{io::AssetSourceBuilder, AssetPath},
    ecs::system::SystemParam,
    prelude::*,
};
use typed_path::Utf8WindowsPath;

use crate::{
    asset_source::vfs::{reader::VfsAssetReader, state::VfsState},
    types::binder::{Archive, ArchiveEntry},
};

#[derive(Resource)]
pub struct VfsStore {
    pub(crate) state: Arc<VfsState>,
    pending: Vec<PendingMount>,
}

struct PendingMount {
    prefix: String,
    handle: Handle<Archive>,
}

#[derive(SystemParam)]
pub struct Vfs<'w> {
    store: ResMut<'w, VfsStore>,
    asset_server: Res<'w, AssetServer>,
}

impl Vfs<'_> {
    pub fn mount_by_handle(&mut self, prefix: impl Into<String>, handle: Handle<Archive>) {
        self.store.pending.push(PendingMount {
            prefix: prefix.into(),
            handle,
        });

        self.store
            .state
            .pending_mounts
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn mount<'a>(&mut self, prefix: impl Into<String>, path: impl Into<AssetPath<'a>>) {
        let path = path.into();
        let handle = self.asset_server.load::<Archive>(path);

        self.mount_by_handle(prefix, handle);
    }

    pub fn load<A: Asset>(&self, path: impl Into<String>) -> Handle<A> {
        self.asset_server.load(format!("vfs://{}", path.into()))
    }
}

pub struct VfsAssetSourcePlugin;

impl Plugin for VfsAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        let state = VfsState::new();
        let reader_state = state.clone();

        app.insert_resource(VfsStore {
            state,
            pending: Vec::new(),
        })
        .add_systems(PreUpdate, bind_archives)
        .register_asset_source(
            "vfs",
            AssetSourceBuilder::new(move || Box::new(VfsAssetReader::new(reader_state.clone()))),
        );
    }
}

fn bind_archives(
    mut store: ResMut<VfsStore>,
    mut assets: ResMut<Assets<Archive>>,
    mut entries: ResMut<Assets<ArchiveEntry>>,
) {
    if store.pending.is_empty() {
        return;
    }

    let state = store.state.clone();
    let mut inner = state.inner.blocking_write();

    store.pending.retain(|mount| {
        let Some(archive) = assets.remove(mount.handle.id()) else {
            return true;
        };

        trace!("mounting {:?}", mount.handle.path());

        for (name, handle) in archive.files {
            let Some(entry) = entries.remove(handle.id()) else {
                continue;
            };

            let windows_path = Utf8WindowsPath::new(&name);
            let path = format!(
                "{}/{}",
                mount.prefix,
                windows_path.file_name().expect("no name")
            );
            trace!("mounted {path}");
            inner.insert_entry(path, entry.data);
        }

        let prev = state.pending_mounts.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            inner.drain_waiters();
        }

        false
    });
}
