use super::attribute_id_manager::AttributeId;
use super::attribute_snapshot::AttributeSnapshot;
use bevy::prelude::*;

#[derive(Component)]
pub struct AttributeSetSnapshot {
    snapshot: Box<[Option<Box<AttributeSnapshot>>]>,
    source_entity: Entity,
}

impl AttributeSetSnapshot {
    pub fn new(snapshot: Vec<Option<Box<AttributeSnapshot>>>, source_entity: Entity) -> Self {
        Self {
            snapshot: snapshot.into_boxed_slice(),
            source_entity,
        }
    }

    pub fn get_current_value(&self, id: AttributeId) -> Option<f64> {
        let index = id.to_index();
        debug_assert!(index < self.snapshot.len());
        if let Some(attr) = &self.snapshot[index] {
            return Some(attr.current());
        }
        None
    }

    pub fn get_base_value(&self, id: AttributeId) -> Option<f64> {
        let index = id.to_index();
        debug_assert!(index < self.snapshot.len());
        if let Some(attr) = &self.snapshot[index] {
            return Some(attr.base());
        }
        None
    }

    pub fn get_source_entity(&self) -> Entity {
        self.source_entity
    }
}
