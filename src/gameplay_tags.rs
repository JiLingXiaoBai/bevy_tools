use bevy::platform::collections::HashMap;
use bevy::prelude::*;

const BLOCK_SIZE: usize = 8; //The number of bits of u64
pub const MAX_TAG_BLOCKS: usize = 8;

pub struct GameplayTagRegistry {
    map: HashMap<String, usize>,
}
#[derive(Component)]
pub struct GameplayTags {
    pub tag_bits: [u64; MAX_TAG_BLOCKS],
}

impl GameplayTagRegistry {
    pub fn register(&mut self, tag: &str) -> usize {
        match self.map.get(tag) {
            Some(id) => *id,
            None => {
                let id = self.map.len();
                if id >= MAX_TAG_BLOCKS * BLOCK_SIZE {
                    panic!("Too many tags, make the MAX_TAG_BLOCKS a litter bigger!")
                }
                self.map.insert(tag.to_string(), id);
                id
            }
        }
    }

    pub fn get(&self, tag: &str) -> Option<usize> {
        self.map.get(tag).copied()
    }
}
