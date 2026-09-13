use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// The Virtual File System (VFS) emulates the block-based storage seen in WebVM.
/// It maps virtual paths to .ure resource identities.
pub struct UniaVFS {
    mounts: Arc<Mutex<HashMap<String, String>>>,
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>, // Simulated IndexedDB storage
}

impl UniaVFS {
    pub fn new() -> Self {
        Self {
            mounts: Arc::new(Mutex::new(HashMap::new())),
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn mount(&self, path: &str, resource_id: &str) {
        let mut mounts = self.mounts.lock().unwrap();
        mounts.insert(path.to_string(), resource_id.to_string());
    }

    pub fn resolve_path(&self, path: &str) -> Option<String> {
        let mounts = self.mounts.lock().unwrap();
        mounts.get(path).cloned()
    }

    pub fn write_file(&self, path: &str, data: Vec<u8>) {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(path.to_string(), data);
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        let storage = self.storage.lock().unwrap();
        storage.get(path).cloned()
    }
}
