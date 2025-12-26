use super::*;
use crate::settings::GameplayAbilitySystemSettings;
use bevy::prelude::*;

pub const ATTRIBUTE_SET_SIZE: usize = GameplayAbilitySystemSettings::ATTRIBUTE_SET_SIZE;

#[derive(Component)]
pub struct AttributeSet {
    attributes: Box<[Option<Box<Attribute>>]>,
}

impl Default for AttributeSet {
    fn default() -> Self {
        let mut attrs = Vec::with_capacity(ATTRIBUTE_SET_SIZE);
        for _ in 0..ATTRIBUTE_SET_SIZE {
            attrs.push(None);
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
        let mut attr = Box::new(Attribute::default());
        attr.init(base_value, executor);
        self.attributes[index] = Some(attr);
    }

    pub fn recalculate_attribute(&mut self, id: AttributeId) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &mut self.attributes[index] {
            attr.recalculate();
        }
    }

    pub fn recalculate_all(&mut self) {
        self.attributes
            .iter_mut()
            .flatten()
            .for_each(|attr| attr.recalculate());
    }

    pub fn get_value(&mut self, id: AttributeId) -> Option<f64> {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &mut self.attributes[index] {
            return Some(attr.get_value());
        }
        None
    }
}
