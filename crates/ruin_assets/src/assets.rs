use std::collections::HashMap;

use crate::asset::{Asset, AssetPath};
use crate::handle::{Handle, Index};

pub struct AssetCache<T: Asset> {
    assets: HashMap<Handle<T>, AssetData<T>>,
    path_lookup: HashMap<AssetPath, Handle<T>>,
    next_index: Index,
}

impl<T: Asset> AssetCache<T> {}

impl<T: Asset> AssetCache<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_lookup: HashMap::new(),
            next_index: 0,
        }
    }

    /// Insert an asset into the cache, optionally associating it with a path.
    pub fn insert(&mut self, asset: T, path: Option<AssetPath>) -> Handle<T> {
        if let Some(p) = &path {
            if let Some(existing) = self.path_lookup.get(p) {
                return *existing;
            }
        }

        let handle = Handle::new(self.next_index);
        self.next_index += 1;

        self.assets.insert(
            handle,
            AssetData {
                asset,
                path: path.clone(),
            },
        );

        if let Some(p) = path {
            self.path_lookup.insert(p, handle);
        }

        handle
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.assets.get(&handle).map(|entry| &entry.asset)
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.assets.get_mut(&handle).map(|entry| &mut entry.asset)
    }

    pub fn get_handle_for_path(&self, path: &AssetPath) -> Option<Handle<T>> {
        self.path_lookup.get(path).copied()
    }
}

pub struct AssetData<T: Asset> {
    pub asset: T,
    pub path: Option<AssetPath>,
    // metadata can go in here
}
