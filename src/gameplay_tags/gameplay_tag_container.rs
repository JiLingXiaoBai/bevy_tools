use super::*;
use bevy::prelude::*;

#[derive(Clone)]
pub struct GameplayTagContainer {
    ref_counts: Box<[u16]>,
    generation: usize,
}

impl GameplayTagContainer {
    pub fn new() -> Self {
        Self {
            ref_counts: Box::new([0; MAX_TAG_COUNTS]),
            generation: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ref_counts.fill(0);
        self.generation += 1;
    }

    pub fn get_generation(&self) -> usize {
        self.generation
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
    pub fn allocate_container(&mut self) -> (usize, usize) {
        if let Some(index) = self.pool_free_list.pop() {
            if self.container_pool.len() > index {
                self.container_pool[index].reset();
                return (index, self.container_pool[index].get_generation());
            }
            panic!("Invalid index {:?} for pool_free_list", index)
        } else {
            let index = self.container_pool.len();
            self.container_pool.push(GameplayTagContainer::new());
            (index, self.container_pool[index].get_generation())
        }
    }

    fn check_index_and_generation(&self, index: usize, generation: usize) {
        let container_len = self.container_pool.len();
        if container_len < index {
            panic!(
                "Invalid index {:?} for container_pool len {:?}",
                index, container_len
            )
        }
        let container_generation = self.container_pool[index].get_generation();
        if container_generation != generation {
            panic!(
                "Invalid generation {:?} for container generation {:?}",
                generation, container_generation
            )
        }
    }

    pub fn free_container(&mut self, index: usize, generation: usize) {
        self.check_index_and_generation(index, generation);
        self.pool_free_list.push(index);
    }

    pub fn get_ref_counts_mut(&mut self, index: usize, generation: usize) -> &mut Box<[u16]> {
        self.check_index_and_generation(index, generation);
        self.container_pool[index].get_mut()
    }
    pub fn get_ref_counts(&self, index: usize, generation: usize) -> &Box<[u16]> {
        self.check_index_and_generation(index, generation);
        self.container_pool[index].get()
    }
}
