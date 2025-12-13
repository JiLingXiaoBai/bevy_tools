use super::*;
use crate::settings::GameplayAbilitySystemSettings;
use bevy::prelude::*;

pub const ATTRIBUTE_SET_SIZE: usize = GameplayAbilitySystemSettings::ATTRIBUTE_SET_SIZE;

#[derive(Component)]
pub struct AttributeSet {
    attributes: [Attribute; ATTRIBUTE_SET_SIZE],
}

impl AttributeSet {
    pub fn initialize_attribute(&mut self, id: AttributeId, base_value: f64, clamp_min: Option<f64>, clamp_max: Option<f64>) {
        let index = id.to_index();
        let attr = &mut self.attributes[index];
        attr.init(base_value, clamp_min, clamp_max);
    }
}