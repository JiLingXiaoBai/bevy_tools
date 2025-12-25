use super::*;
use crate::settings::GameplayAbilitySystemSettings;
use bevy::prelude::*;

pub const ATTRIBUTE_SET_SIZE: usize = GameplayAbilitySystemSettings::ATTRIBUTE_SET_SIZE;

#[derive(Component)]
pub struct AttributeSet {
    attributes: Box<[Attribute]>,
    _aggregator: Vec<Aggregator>,
}

impl Default for AttributeSet {
    fn default() -> Self {
        Self {
            attributes: Box::new([Attribute::default(); ATTRIBUTE_SET_SIZE]),
            _aggregator: vec![Aggregator::default(); ATTRIBUTE_SET_SIZE],
        }
    }
}

impl AttributeSet {
    pub fn initialize_attribute(&mut self, id: AttributeId, base_value: f64) {
        let index = id.to_index();
        if index >= ATTRIBUTE_SET_SIZE {
            panic!("Exceeded ATTRIBUTE_SET_SIZE")
        }
        let attr = &mut self.attributes[index];
        attr.init(base_value);
    }
}
