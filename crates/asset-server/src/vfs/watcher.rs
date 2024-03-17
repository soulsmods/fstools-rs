use std::{sync::Arc, thread};

use bevy::asset::io::{AssetSourceEvent, AssetWatcher};
use crossbeam_channel::{Receiver, Sender};

use crate::vfs::VfsEvent;

impl AssetWatcher for VfsWatcher {}

pub struct VfsWatcher {
    vfs_event_receiver: Receiver<VfsEvent>,
    asset_event_receiver: Sender<AssetSourceEvent>,
    keepalive: Arc<()>,
}

impl VfsWatcher {
    pub fn new(rx: Receiver<VfsEvent>, tx: Sender<AssetSourceEvent>) -> Self {
        Self {
            vfs_event_receiver: rx,
            asset_event_receiver: tx,
            keepalive: Arc::default(),
        }
    }

    pub fn start(&mut self) {
        let tx = self.asset_event_receiver.clone();
        let rx = self.vfs_event_receiver.clone();
        let recv = Arc::downgrade(&self.keepalive);

        thread::spawn(move || {
            while recv.upgrade().is_some() {
                match rx.recv() {
                    Ok(VfsEvent::Added(path)) => {
                        tx.send(AssetSourceEvent::AddedAsset(path.clone()))
                            .and_then(|_| tx.send(AssetSourceEvent::ModifiedAsset(path.clone())))
                            .expect("failed to notify asset watcher");
                    }
                    _ => break,
                }
            }
        });
    }
}
