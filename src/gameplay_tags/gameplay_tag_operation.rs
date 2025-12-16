use bevy::prelude::*;
use super::*;
#[derive(Message)]
pub struct GameplayTagOperation {
    pub entity: Entity,
    pub tags_to_add: Vec<GameplayTag>,
    pub tags_to_remove: Vec<GameplayTag>,
}
pub fn process_tag_operations(
    manager: Res<GameplayTagManager>,
    mut container_pool: ResMut<GameplayTagContainerPool>,
    mut components: Query<&mut GameplayTagComponent>,
    mut operations: MessageReader<GameplayTagOperation>,
) {
    for operation in operations.read() {
        if let Ok(mut tag_comp) = components.get_mut(operation.entity) {
            let container_index = tag_comp.get_container_index();
            let tag_comp_bits = tag_comp.get_bit_set_mut();
            let ref_counts = container_pool.get_ref_counts_mut(container_index);
            for tag in &operation.tags_to_add {
                if let Some(inherited_bits) = manager.get_inherited_bits(tag) {
                    for block_index in 0..MAX_TAG_BLOCKS {
                        let base_index = (block_index * TAG_BITS_PER_BLOCK) as u16;
                        let mut current_block = inherited_bits[block_index];

                        while current_block != 0 {
                            let bit_offset = current_block.trailing_zeros();
                            let current_index = base_index + bit_offset as u16;
                            let index_usize = current_index as usize;
                            if index_usize < ref_counts.len() {
                                ref_counts[index_usize] = ref_counts[index_usize].saturating_add(1);
                            }
                            current_block &= !(1u64 << bit_offset);
                        }
                    }

                    for i in 0..MAX_TAG_BLOCKS {
                        tag_comp_bits[i] |= inherited_bits[i];
                    }
                }
            }

            let mut bits_to_clear = GameplayTagBits::default();

            for tag in &operation.tags_to_remove {
                let tag_bit_index = tag.get_bit_index_u16();
                let tag_index_usize = tag_bit_index as usize;

                if tag_index_usize < ref_counts.len() && ref_counts[tag_index_usize] > 0 {
                    if manager.check_has_active_descendants(tag_bit_index, ref_counts) {
                        continue;
                    }

                    if let Some(inherited_bits) = manager.get_inherited_bits(tag) {
                        for block_index in 0..MAX_TAG_BLOCKS {
                            let base_index = (block_index * TAG_BITS_PER_BLOCK) as u16;
                            let mut current_block = inherited_bits[block_index];

                            while current_block != 0 {
                                let bit_offset = current_block.trailing_zeros();
                                let current_index = base_index + bit_offset as u16;
                                let index_usize = current_index as usize;

                                if index_usize < ref_counts.len() {
                                    ref_counts[index_usize] = ref_counts[index_usize].saturating_sub(1);

                                    if ref_counts[index_usize] == 0 {
                                        let block = index_usize >> BLOCK_SIZE_EXPONENT;
                                        let bit = index_usize & (TAG_BITS_PER_BLOCK - 1);
                                        bits_to_clear[block] |= 1u64 << bit;
                                    }
                                }

                                current_block &= !(1u64 << bit_offset);
                            }
                        }
                    }
                }
            }

            for i in 0..MAX_TAG_BLOCKS {
                tag_comp_bits[i] &= !bits_to_clear[i];
            }
        }
    }
}