use super::AbilitySpecHandle;
use bevy::prelude::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveAbilityHandle(u32);

impl ActiveAbilityHandle {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get_value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityActivationStatus {
    Activating,
    Active,
    Ending,
    Cancelled,
}

pub struct ActiveGameplayAbility {
    handle: ActiveAbilityHandle,
    spec_handle: AbilitySpecHandle,
    target: Entity,
    status: AbilityActivationStatus,
}

impl ActiveGameplayAbility {
    pub fn new(
        handle: ActiveAbilityHandle,
        spec_handle: AbilitySpecHandle,
        target: Entity,
        status: AbilityActivationStatus,
    ) -> Self {
        Self {
            handle,
            spec_handle,
            target,
            status,
        }
    }

    pub fn get_handle(&self) -> ActiveAbilityHandle {
        self.handle
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
