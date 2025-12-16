use super::*;
use bevy::prelude::*;

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

#[derive(Resource)]
pub struct GameplayTagContainerPool {
    container_pool: Vec<GameplayTagContainer>,
    pool_free_list: Vec<usize>,
}

impl Default for GameplayTagContainerPool {
    fn default() -> Self {
        Self {
            container_pool: Vec::new(),
            pool_free_list: Vec::new(),
        }
    }
}

impl GameplayTagContainerPool {
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
