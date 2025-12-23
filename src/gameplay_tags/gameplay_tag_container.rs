use super::*;
use bevy::prelude::{Component, Res};
use bevy::render::render_resource::encase::private::RuntimeSizedArray;

pub const BLOCK_SIZE_EXPONENT: usize = 6; // 2^6 =64
pub const TAG_BITS_PER_BLOCK: usize = 64;
pub const MAX_TAG_BLOCKS: usize = (MAX_TAG_COUNTS + TAG_BITS_PER_BLOCK - 1) / TAG_BITS_PER_BLOCK;

pub type GameplayTagBits = [u64; MAX_TAG_BLOCKS];

pub fn tag_bits_from_tags(tags: &[GameplayTag]) -> GameplayTagBits {
    let mut result = GameplayTagBits::default();
    for tag in tags {
        add_bit_with_tag(&mut result, tag);
    }
    result
}

pub fn add_bit_with_tag(bits: &mut GameplayTagBits, tag: &GameplayTag) {
    let tag_bit_index = tag.get_bit_index_usize();
    if tag_bit_index >= MAX_TAG_COUNTS {
        panic!("Exceeded MAX_TAG_COUNTS");
    }
    let block = tag_bit_index >> BLOCK_SIZE_EXPONENT;
    let bit = tag_bit_index & (TAG_BITS_PER_BLOCK - 1);
    bits[block] |= 1u64 << bit;
}

#[derive(Component)]
pub struct GameplayTagContainer {
    tag_bits: GameplayTagBits,
    ref_counts: Box<[u16]>,
}

impl Default for GameplayTagContainer {
    fn default() -> Self {
        Self {
            tag_bits: GameplayTagBits::default(),
            ref_counts: Box::new([0; MAX_TAG_COUNTS]),
        }
    }
}

impl GameplayTagContainer {
    /// Adds a tag, incrementing reference counts for itself and all parents, and updating the Bitset.
    pub fn add_tag(&mut self, tag: &GameplayTag, manager: &Res<GameplayTagManager>) {
        if let Some(inherited_bits) = manager.get_inherited_bits(tag) {
            // 1. Update Reference Counts (for self and all parents)
            for block_index in 0..MAX_TAG_BLOCKS {
                let base_index = (block_index * TAG_BITS_PER_BLOCK) as u16;
                let mut current_block = inherited_bits[block_index];

                while current_block != 0 {
                    let bit_offset = current_block.trailing_zeros();
                    let current_index = base_index + bit_offset as u16;
                    let index_usize = current_index as usize;
                    if index_usize < self.ref_counts.len() {
                        self.ref_counts[index_usize] =
                            self.ref_counts[index_usize].saturating_add(1);
                    }
                    current_block &= !(1u64 << bit_offset);
                }
            }

            // 2. Update Bitset (OR operation)
            for i in 0..MAX_TAG_BLOCKS {
                self.tag_bits[i] |= inherited_bits[i];
            }
        }
    }
    /// Removes a tag, decrementing reference counts. Clears the bit only if the count drops to zero.
    pub fn remove_tag(&mut self, tag: &GameplayTag, manager: &Res<GameplayTagManager>) {
        // Only proceed if the tag was explicitly present (count > 0 for this exact tag)
        let tag_bit_index = tag.get_bit_index_usize();
        if tag_bit_index < self.ref_counts.len() && self.ref_counts[tag_bit_index] > 0 {
            if manager.check_has_active_descendants(tag_bit_index, &self.ref_counts) {
                return;
            }

            if let Some(inherited_bits) = manager.get_inherited_bits(tag) {
                // 1. Update Reference Counts and track which bits need to be cleared
                let mut bits_to_clear = [0u64; MAX_TAG_BLOCKS];

                for block_index in 0..MAX_TAG_BLOCKS {
                    let base_index = (block_index * TAG_BITS_PER_BLOCK) as u16;
                    let mut current_block = inherited_bits[block_index];

                    while current_block != 0 {
                        let bit_offset = current_block.trailing_zeros();
                        let current_index = (base_index as u32 + bit_offset) as usize;
                        if current_index < self.ref_counts.len() {
                            self.ref_counts[current_index] =
                                self.ref_counts[current_index].saturating_sub(1);
                            if self.ref_counts[current_index] == 0 {
                                // mark this bit should be reset int Bitset
                                let block = current_index >> BLOCK_SIZE_EXPONENT;
                                let bit = current_index & (TAG_BITS_PER_BLOCK - 1);
                                bits_to_clear[block] |= 1u64 << bit;
                            }
                        }
                        current_block &= !(1u64 << bit_offset);
                    }
                }

                // 2. Update Bitset (AND NOT operation based on zero counts)
                for i in 0..MAX_TAG_BLOCKS {
                    self.tag_bits[i] &= !bits_to_clear[i];
                }
            }
        }
    }

    pub fn add_tags(&mut self, tags: &[GameplayTag], manager: &Res<GameplayTagManager>) {
        for tag in tags {
            self.add_tag(tag, manager);
        }
    }

    pub fn remove_tags(&mut self, tags: &[GameplayTag], manager: &Res<GameplayTagManager>) {
        for tag in tags {
            self.remove_tag(tag, manager);
        }
    }
    pub fn has_tag(&self, tag: &GameplayTag) -> bool {
        let tag_bit_index = tag.get_bit_index_usize();
        if tag_bit_index >= MAX_TAG_COUNTS {
            panic!("Exceeded MAX_TAG_COUNTS");
        };
        let block = tag_bit_index >> BLOCK_SIZE_EXPONENT;
        let bit = tag_bit_index & (TAG_BITS_PER_BLOCK - 1);
        (self.tag_bits[block] & (1u64 << bit)) != 0
    }
    pub fn has_all(&self, tags: &[GameplayTag]) -> bool {
        let tag_bits = tag_bits_from_tags(tags);
        for i in 0..MAX_TAG_BLOCKS {
            if (self.tag_bits[i] & tag_bits[i]) != tag_bits[i] {
                return false;
            }
        }
        true
    }
    pub fn has_any(&self, tags: &[GameplayTag]) -> bool {
        let tag_bits = tag_bits_from_tags(tags);
        for i in 0..MAX_TAG_BLOCKS {
            if (self.tag_bits[i] & tag_bits[i]) != 0 {
                return true;
            }
        }
        false
    }
}
