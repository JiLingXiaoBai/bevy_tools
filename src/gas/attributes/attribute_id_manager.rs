use super::ATTRIBUTE_SET_SIZE;
use crate::{UniqueName, UniqueNamePool};
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::prelude::{ResMut, Resource};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeId(u16);

impl AttributeId {
    pub(crate) fn new(index: u16) -> Self {
        Self(index)
    }
    pub fn to_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeIdError {
    CapacityExceeded { max: usize },
}

impl fmt::Display for AttributeIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeIdError::CapacityExceeded { max } => {
                write!(f, "attribute id capacity exceeded; max attributes: {max}")
            }
        }
    }
}

impl Error for AttributeIdError {}

#[derive(Resource)]
pub struct AttributeIdManager {
    name_to_index: HashMap<UniqueName, u16>,
    next_id_index: u16,
}

impl Default for AttributeIdManager {
    fn default() -> Self {
        Self {
            name_to_index: HashMap::new(),
            next_id_index: 0,
        }
    }
}

impl AttributeIdManager {
    pub fn get_attribute_id(&self, unique_name: UniqueName) -> Option<AttributeId> {
        self.name_to_index
            .get(&unique_name)
            .map(|&id| AttributeId::new(id))
    }

    pub fn register_id_internal(
        &mut self,
        unique_name: UniqueName,
    ) -> Result<AttributeId, AttributeIdError> {
        if let Some(&index) = self.name_to_index.get(&unique_name) {
            return Ok(AttributeId::new(index));
        }

        let new_index = self.next_id_index;
        if new_index as usize >= ATTRIBUTE_SET_SIZE {
            return Err(AttributeIdError::CapacityExceeded {
                max: ATTRIBUTE_SET_SIZE,
            });
        }

        let attribute_id = AttributeId::new(new_index);
        self.name_to_index.insert(unique_name, new_index);
        self.next_id_index += 1;
        Ok(attribute_id)
    }
}
#[derive(SystemParam)]
pub struct AttributeIdRegister<'w> {
    unique_name_pool: ResMut<'w, UniqueNamePool>,
    attribute_id_manager: ResMut<'w, AttributeIdManager>,
}

impl<'w> AttributeIdRegister<'w> {
    pub fn request_or_register_attribute_id(
        &mut self,
        attribute_id_name: &str,
    ) -> Result<AttributeId, AttributeIdError> {
        let unique_name = self.unique_name_pool.new_name(attribute_id_name);

        if let Some(attribute_id) = self.attribute_id_manager.get_attribute_id(unique_name) {
            return Ok(attribute_id);
        }
        self.attribute_id_manager.register_id_internal(unique_name)
    }
}
