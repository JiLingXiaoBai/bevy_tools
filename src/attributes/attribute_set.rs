use super::*;
use crate::settings::GameplayAbilitySystemSettings;
use bevy::prelude::*;

pub const ATTRIBUTE_SET_SIZE: usize = GameplayAbilitySystemSettings::ATTRIBUTE_SET_SIZE;

#[derive(Component)]
pub struct AttributeSet {
    attributes: Box<[Attribute]>,
}

impl Default for AttributeSet {
    fn default() -> Self {
        let mut attrs = Vec::with_capacity(ATTRIBUTE_SET_SIZE);
        for _ in 0..ATTRIBUTE_SET_SIZE {
            attrs.push(Attribute::default());
        }
        Self {
            attributes: attrs.into_boxed_slice(),
        }
    }
}

impl AttributeSet {
    pub fn initialize_attribute(
        &mut self,
        id: AttributeId,
        base_value: f64,
        executor: Option<fn(&Aggregator, f64) -> f64>,
    ) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        let attr = &mut self.attributes[index];
        attr.init(base_value, executor);
    }

    pub fn recalculate_attribute(&mut self, id: AttributeId) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        self.attributes[index].recalculate();
    }

    pub fn recalculate_all(&mut self) {
        for attr in self.attributes.iter_mut() {
            attr.recalculate();
        }
    }
}
