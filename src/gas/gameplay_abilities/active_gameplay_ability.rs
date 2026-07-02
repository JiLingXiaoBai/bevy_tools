use super::AbilitySpecHandle;
use bevy::prelude::{Component, Entity};

pub type ActiveAbilityHandle = Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityActivationStatus {
    Activating,
    Active,
    Ending,
    Cancelled,
}

#[derive(Component, Clone)]
pub struct ActiveGameplayAbility {
    source: Entity,
    spec_handle: AbilitySpecHandle,
    target: Entity,
    status: AbilityActivationStatus,
}

impl ActiveGameplayAbility {
    pub fn new(
        source: Entity,
        spec_handle: AbilitySpecHandle,
        target: Entity,
        status: AbilityActivationStatus,
    ) -> Self {
        Self {
            source,
            spec_handle,
            target,
            status,
        }
    }

    pub fn get_source(&self) -> Entity {
        self.source
    }

    pub fn get_spec_handle(&self) -> AbilitySpecHandle {
        self.spec_handle
    }

    pub fn get_target(&self) -> Entity {
        self.target
    }

    pub fn get_status(&self) -> AbilityActivationStatus {
        self.status
    }

    pub fn set_status(&mut self, status: AbilityActivationStatus) {
        self.status = status;
    }
}
