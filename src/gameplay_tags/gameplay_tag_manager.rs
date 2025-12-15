use super::*;
use crate::settings::GameplayAbilitySystemSettings;
use crate::unique_name::UniqueName;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
pub const MAX_TAG_COUNTS: usize = GameplayAbilitySystemSettings::GAMEPLAY_TAG_SIZE;
#[derive(Resource)]
pub struct GameplayTagManager {
    tag_name_to_index: HashMap<UniqueName, u16>,
    tag_parent_index: Vec<Option<u16>>,
    tag_children: Vec<Vec<u16>>,
    tag_inherited_bits: Vec<GameplayTagBits>,
    next_tag_index: u16,
    container_pool: Vec<GameplayTagContainer>,
    pool_free_list: Vec<usize>,
}

impl Default for GameplayTagManager {
    fn default() -> Self {
        Self {
            tag_name_to_index: HashMap::new(),
            tag_parent_index: Vec::new(),
            tag_children: Vec::new(),
            tag_inherited_bits: Vec::new(),
            next_tag_index: 0,
            container_pool: Vec::new(),
            pool_free_list: Vec::new(),
        }
    }
}

impl GameplayTagManager {
    pub fn get_tag(&self, unique_name: UniqueName) -> Option<GameplayTag> {
        self.tag_name_to_index
            .get(&unique_name)
            .map(|&index| GameplayTag::new(index))
    }

    pub fn register_tag_internal(
        &mut self,
        unique_name: UniqueName,
        parent_tag_index: Option<u16>,
    ) -> GameplayTag {
        if let Some(&index) = self.tag_name_to_index.get(&unique_name) {
            return GameplayTag::new(index);
        }

        let new_index = self.next_tag_index;
        if new_index as usize >= MAX_TAG_COUNTS {
            panic!("Exceeded MAX_TAG_COUNTS");
        }

        // Create inherited bits: Start with parent's bits or new empty bits
        let mut inherited_bits = parent_tag_index
            .and_then(|p_index| self.tag_inherited_bits.get(p_index as usize).cloned())
            .unwrap_or_else(GameplayTagBits::default);

        // Set the current tag's own bit in the inherited bits
        let self_tag = GameplayTag::new(new_index);
        add_bit_with_tag(&mut inherited_bits, &self_tag);

        // Update the Manager data structures
        if new_index as usize == self.tag_parent_index.len() {
            self.tag_parent_index.push(parent_tag_index);
            self.tag_inherited_bits.push(inherited_bits);
            self.tag_children.push(Vec::new());
        }
        if let Some(p_index) = parent_tag_index {
            if (p_index as usize) < self.tag_children.len() {
                self.tag_children[p_index as usize].push(new_index);
            }
        }
        self.next_tag_index += 1;

        self_tag
    }

    pub fn get_inherited_bits(&self, tag: &GameplayTag) -> Option<&GameplayTagBits> {
        self.tag_inherited_bits.get(tag.get_bit_index_usize())
    }
    pub fn check_has_active_descendants(&self, tag_index: u16, ref_counts: &Box<[u16]>) -> bool {
        let mut stack: Vec<u16> = Vec::new();
        if (tag_index as usize) < self.tag_children.len() {
            stack.extend(self.tag_children[tag_index as usize].iter().copied());
        } else {
            return false;
        }

        while let Some(current_index) = stack.pop() {
            let index_usize = current_index as usize;
            if index_usize < ref_counts.len() && ref_counts[index_usize] > 0 {
                return true;
            }

            if (index_usize) < self.tag_children.len() {
                if let Some(children) = self.tag_children.get(index_usize) {
                    stack.extend(children.iter().copied());
                }
            }
        }
        false
    }

    pub fn allocate_container(&mut self) -> usize {
        if let Some(index) = self.pool_free_list.pop() {
            self.container_pool[index] = GameplayTagContainer::new();
            index
        } else {
            let index = self.container_pool.len();
            self.container_pool.push(GameplayTagContainer::new());
            index
        }
    }

    pub fn free_container(&mut self, index: usize) {
        self.pool_free_list.push(index);
    }

    pub fn get_ref_counts_mut(&mut self, index: usize) -> &mut Box<[u16]> {
        if index < self.container_pool.len() {
            self.container_pool[index].get_mut()
        } else {
            panic!(
                "Invalid ref_count_index {} for len {:?}",
                index,
                self.container_pool.len()
            );
        }
    }
    pub fn get_ref_counts(&self, index: usize) -> &Box<[u16]> {
        if index < self.container_pool.len() {
            self.container_pool[index].get()
        } else {
            panic!(
                "Invalid ref_count_index {} for len {:?}",
                index,
                self.container_pool.len()
            );
        }
    }
}

#[derive(Clone)]
pub struct GameplayTagContainer {
    ref_counts: Box<[u16]>,
}

impl GameplayTagContainer {
    pub fn new() -> Self {
        Self {
            ref_counts: Box::new([0; MAX_TAG_COUNTS]),
        }
    }
    pub fn get_mut(&mut self) -> &mut Box<[u16]> {
        &mut self.ref_counts
    }

    pub fn get(&self) -> &Box<[u16]> {
        &self.ref_counts
    }
}
