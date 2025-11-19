use core::hash::{BuildHasher, Hash, Hasher};

use bevy::platform::collections::HashMap;
use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
fn compute_hash(input: &str) -> u64 {
    let mut hasher = FixedHasher::default().build_hasher();
    input.hash(&mut hasher);
    hasher.finish()
}

#[derive(Resource, Default)]
pub struct UniqueNamePool {
    map: HashMap<u64, String>,
}

impl UniqueNamePool {
    fn get_or_insert(&mut self, name: &str) -> u64 {
        if name.is_empty() {
            return 0;
        }
        let mut hash = compute_hash(name);
        if let Some(existing) = self.map.get(&hash) {
            if existing != name {
                let original = hash;
                loop {
                    hash = hash.wrapping_add(1);
                    if !self.map.contains_key(&hash) {
                        break;
                    }
                }
                error!(
                    "Hash collision: '{}' vs '{}' hash={}, {} rehashed to {}",
                    existing, name, original, name, hash
                );
            }
            return hash;
        }

        self.map.insert(hash, name.to_string());
        hash
    }

    fn get(&self, hash: u64) -> &str {
        if hash == 0 {
            ""
        } else {
            self.map.get(&hash).map(|s| s.as_str()).unwrap_or("")
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueName {
    pub comparison_index: u64,
}

impl UniqueName {
    pub fn new(name: &str, pool: &mut UniqueNamePool) -> Self {
        let comparison_index = { pool.get_or_insert(name) };
        Self { comparison_index }
    }

    pub fn as_str<'a>(&self, pool: &'a UniqueNamePool) -> &'a str {
        pool.get(self.comparison_index)
    }
}
