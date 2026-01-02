use super::active_gameplay_effect::ActiveEffects;
use crate::attributes::{AttributeId, AttributeSet};
use crate::gameplay_tags::GameplayTagContainer;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Query;
use bevy::ecs::world::Mut;

pub struct EffectContext<'w, 's> {
    pub source: Option<Entity>,
    pub target: Option<Entity>,
    pub attr_set_query: &'w mut Query<'w, 's, &'static mut AttributeSet>,
    pub tag_container_query: &'w mut Query<'w, 's, &'static mut GameplayTagContainer>,
    pub active_effects_query: &'w mut Query<'w, 's, &'static mut ActiveEffects>,
    pub level: u32,
}

pub enum EffectContextEntityType {
    Source,
    Target,
}

impl<'w, 's> EffectContext<'w, 's> {
    pub fn get_attribute_current_value(
        &self,
        entity_type: EffectContextEntityType,
        id: AttributeId,
    ) -> Option<f64> {
        let entity = match entity_type {
            EffectContextEntityType::Source => self.source,
            EffectContextEntityType::Target => self.target,
        };

        entity
            .and_then(|ent| self.attr_set_query.get(ent).ok())
            .and_then(|attr_set| attr_set.get_current_value(id))
    }

    pub fn get_attr_set_mut(
        &mut self,
        entity_type: EffectContextEntityType,
    ) -> Option<Mut<'_, AttributeSet>> {
        let entity = match entity_type {
            EffectContextEntityType::Source => self.source,
            EffectContextEntityType::Target => self.target,
        };

        entity.and_then(|ent| self.attr_set_query.get_mut(ent).ok())
    }

    pub fn get_tag_container_mut(
        &mut self,
        entity_type: EffectContextEntityType,
    ) -> Option<Mut<'_, GameplayTagContainer>> {
        let entity = match entity_type {
            EffectContextEntityType::Source => self.source,
            EffectContextEntityType::Target => self.target,
        };

        entity.and_then(|ent| self.tag_container_query.get_mut(ent).ok())
    }

    pub fn get_active_effects_mut(
        &mut self,
        entity_type: EffectContextEntityType,
    ) -> Option<Mut<'_, ActiveEffects>> {
        let entity = match entity_type {
            EffectContextEntityType::Source => self.source,
            EffectContextEntityType::Target => self.target,
        };
        entity.and_then(|ent| self.active_effects_query.get_mut(ent).ok())
    }
}
