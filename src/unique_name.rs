use bevy::platform::collections::HashMap;
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
use core::fmt;
use core::hash::{BuildHasher, Hash, Hasher};
fn compute_hash(input: &str) -> u64 {
    let mut hasher = FixedHasher::default().build_hasher();
    input.hash(&mut hasher);
    hasher.finish()
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueName(u32);
impl fmt::Debug for UniqueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UniqueName({})", self.0)
    }
}

#[derive(Resource)]
pub struct UniqueNamePool {
    entry_pool: Vec<String>,
    lookup_hash: HashMap<u64, u32>,
}
impl Default for UniqueNamePool {
    fn default() -> Self {
        let mut pool = Self {
            entry_pool: Vec::new(),
            lookup_hash: HashMap::new(),
        };
        pool.entry_pool.push("".to_string());
        pool
    }
}

impl UniqueNamePool {
    fn get_or_insert(&mut self, name: &str) -> u32 {
        if name.is_empty() {
            return 0;
        }
        let hash = compute_hash(name);
        if let Some(&index) = self.lookup_hash.get(&hash) {
            if cfg!(debug_assertions) {
                if self.entry_pool.get(index as usize).map(|s| s.as_str()) == Some(name) {
                    return index;
                } else {
                    let existing_str = self
                        .entry_pool
                        .get(index as usize)
                        .map(|s| s.as_str())
                        .unwrap_or("UNKNOWN");
                    error!(
                        "FATAL HASH COLLISION: '{}' vs '{}'. Hash: {}. UniqueName system failed.",
                        existing_str, name, hash
                    );
                    panic!("UniqueName Hash Collision Detected in Debug Mode!");
                }
            }
            return index;
        }
        let new_index = self.entry_pool.len() as u32;
        if new_index == u32::MAX {
            panic!("UniqueNamePool capacity exceeded (u32::MAX)");
        }
        self.entry_pool.push(name.to_string());
        self.lookup_hash.insert(hash, new_index);
        new_index
    }

    pub fn new_name(&mut self, name: &str) -> UniqueName {
        UniqueName {
            0: self.get_or_insert(name),
        }
    }

    pub fn get_display_str(&self, name: &UniqueName) -> &str {
        self.entry_pool
            .get(name.0 as usize)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn clear(&mut self) {
        self.lookup_hash.clear();
        self.entry_pool.truncate(1); // only keep the first element: ""
    }
}
