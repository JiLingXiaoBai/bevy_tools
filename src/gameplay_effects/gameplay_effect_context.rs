use crate::ability_system::AbilitySystemComponent;
use crate::attributes::{AttributeId, AttributeSet};
use crate::gameplay_tags::GameplayTagContainer;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Query;

pub struct EffectContext<'a, 'w, 's> {
    pub source: Option<Entity>,
    pub target: Option<Entity>,
    pub attr_set_query: &'a Query<'w, 's, &'static AttributeSet>,
    pub tag_container_query: &'a Query<'w, 's, &'static GameplayTagContainer>,
    pub asc_query: &'a Query<'w, 's, &'static AbilitySystemComponent>,
    pub level: u32,
}

pub enum EffectContextEntityType {
    Source,
    Target,
}

impl<'a, 'w, 's> EffectContext<'a, 'w, 's> {
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
}
