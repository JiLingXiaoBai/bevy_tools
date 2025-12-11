use super::attribute::Attribute;
use bevy::prelude::*;
pub const ATTRIBUTE_SET_SIZE: usize = 256;

#[derive(Component)]
pub struct AttributeSet {
    pub attributes: [Attribute; ATTRIBUTE_SET_SIZE],
}
