use crate::apply_gameplay_effect;
use crate::attributes::{AttributeSet, AttributeSetSnapshot};
use crate::gameplay_abilities::GameplayAbility;
use crate::gameplay_effects::EffectContext;
use crate::gameplay_tags::{GameplayTagContainer, GameplayTagManager};
use crate::randoms::Random;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(SystemParam)]
pub struct AbilitySystemParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub tag_manager: Res<'w, GameplayTagManager>,
    pub random_gen: ResMut<'w, Random>,
    pub attr_set_query: Query<'w, 's, &'static mut AttributeSet>,
    pub tag_container_query: Query<'w, 's, &'static mut GameplayTagContainer>,
    pub asc_query: Query<'w, 's, &'static mut AbilitySystemComponent>,
    pub attr_set_snapshot_query: Query<'w, 's, &'static AttributeSetSnapshot>,
    pub time: Res<'w, Time>,
}

#[derive(Component, Default)]
pub struct AbilitySystemComponent;

pub fn try_activate_ability(
    source: Entity,
    target: Entity,
    ability: &Arc<GameplayAbility>,
    params: &mut AbilitySystemParams,
) -> bool {
    let source_tags = params.tag_container_query.get(source).ok();
    if let Some(tags) = source_tags {
        // 1. Check Tags (Owner Tags / Activation Tags)
        let ability_tags = ability.get_tags();
        if tags.has_any(ability_tags.get_activation_blocked_tags()) {
            return false;
        }
        if !tags.has_all(ability_tags.get_activation_required_tags()) {
            return false;
        }

        // 2. Check Cooldown
        if let Some(cooldown_def) = ability.get_cooldown()
            && tags.has_any(cooldown_def.get_tags().get_granted_tags())
        {
            return false;
        }
    }

    if let Some(cost_def) = &ability.get_cost() {
        // 3. Check Cost
        let context = EffectContext {
            source: Some(source),
            target: Some(target),
            attr_set_query: &params.attr_set_query.as_readonly(),
            tag_container_query: &params.tag_container_query.as_readonly(),
            asc_query: &params.asc_query.as_readonly(),
            attr_set_snapshot: params.attr_set_snapshot_query.get(source).ok(),
            level: ability.get_level(),
        };

        let cost_spec = cost_def.make_spec(&context);
        if let Ok(attr_set) = params.attr_set_query.get(target) {
            for cost in cost_spec.get_modifier_specs() {
                let current_val = attr_set.get_current_value(cost.get_id()).unwrap_or(0.0);
                if current_val + cost.get_value() < 0.0 {
                    return false;
                }
            }
        }
    }
    // 4. Apply Cost
    if let Some(cost_def) = &ability.get_cost() {
        apply_gameplay_effect(source, source, cost_def, params, ability.get_level());
    }

    // 5. Apply Cooldown
    if let Some(cooldown_def) = &ability.get_cooldown() {
        apply_gameplay_effect(source, source, cooldown_def, params, ability.get_level());
    }

    // 6. TODO: Apply Ability
    // apply_gameplay_effect(
    //     source,
    //     target,
    //     ability.get_effect(),
    //     params,
    //     ability.get_level(),
    // );

    true
}
