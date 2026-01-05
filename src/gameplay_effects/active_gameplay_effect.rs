use super::gameplay_effect::{EffectContext, GameplayEffect};
use super::gameplay_effect_spec::{EffectDurationSpec, GameplayEffectSpec};
use crate::ability_system::AbilitySystemParams;
use crate::attributes::AttributeSet;
use crate::gameplay_tags::{GameplayTagContainer, GameplayTagManager};
use bevy::prelude::*;
use std::sync::Arc;

pub type ActiveEffectHandle = Entity;

#[derive(Component)]
pub struct ActiveGameplayEffect {
    spec: GameplayEffectSpec,
    _source: Entity,
    target: Entity,
}

#[derive(Component)]
pub struct ActiveEffectDuration {
    remain_ticks: u32,
}

#[derive(Component)]
pub struct ActiveEffectPeriod {
    period_ticks: u32,
    current_tick: u32,
}

pub fn apply_gameplay_effect(
    source: Entity,
    target: Entity,
    effect_def: &Arc<GameplayEffect>,
    params: &mut AbilitySystemParams,
    level: u32,
) {
    let probability = effect_def.get_probability_to_apply();
    if probability < 1.0 && !params.random_gen.random_bool(probability) {
        return;
    }

    let tags = effect_def.get_tags();

    let target_tags = params.tag_container_query.get(target).unwrap();
    if !target_tags.has_all(tags.get_required_tags())
        || target_tags.has_any(tags.get_blocked_tags())
    {
        return;
    }

    let spec = {
        let context = EffectContext {
            source: Some(source),
            target: Some(target),
            attr_set_query: &params.attr_set_query.as_readonly(),
            tag_container_query: &params.tag_container_query.as_readonly(),
            asc_query: &params.asc_query.as_readonly(),
            attr_set_snapshot: params.attr_set_snapshot_query.get(source).ok(),
            level,
        };

        effect_def.make_spec(&context)
    };
    let duration_spec = spec.get_duration_spec();

    // Instant Effect
    if duration_spec.is_instant() {
        let mut target_attrs_mut = params.attr_set_query.get_mut(target).unwrap();
        for mod_spec in spec.get_modifier_specs() {
            target_attrs_mut.apply_instant_modifier(mod_spec);
        }
        return;
    }

    // Duration Effect -> Spawn Entity
    let mut entity_cmds = params.commands.spawn(ActiveGameplayEffect {
        _source: source,
        target,
        spec: spec.clone(),
    });

    let effect_entity = entity_cmds.id();

    // Add Duration Component
    if let EffectDurationSpec::Duration(duration) = duration_spec
        && *duration > 0
    {
        entity_cmds.insert(ActiveEffectDuration {
            remain_ticks: *duration,
        });
    }

    // Add Period Component
    if let Some(period_spec) = spec.get_period_spec() {
        let period = period_spec.get_period();
        let execute_on_application = period_spec.get_execute_on_applied();
        if execute_on_application {
            let mut target_attrs_mut = params.attr_set_query.get_mut(target).unwrap();
            for mod_spec in spec.get_modifier_specs() {
                target_attrs_mut.apply_instant_modifier(mod_spec);
            }
        }
        if period > 0 {
            entity_cmds.insert(ActiveEffectPeriod {
                period_ticks: period,
                current_tick: 0,
            });
        }
    }

    // Set Parent
    entity_cmds.set_parent_in_place(target);

    // Apply Modifiers to AttributeSet (using Entity ID)
    let mut target_attrs_mut = params.attr_set_query.get_mut(target).unwrap();
    for mod_spec in spec.get_modifier_specs() {
        target_attrs_mut.apply_duration_modifier(mod_spec, effect_entity);
    }

    // Add Tags
    let mut target_tags = params.tag_container_query.get_mut(target).unwrap();
    target_tags.add_tags(tags.get_granted_tags(), &params.tag_manager);
}

pub fn tick_effect_duration_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActiveEffectDuration, &ActiveGameplayEffect)>,
    mut attr_query: Query<&mut AttributeSet>,
    mut tag_query: Query<&mut GameplayTagContainer>,
    tag_manager: Res<GameplayTagManager>,
) {
    for (entity, mut duration, effect) in query.iter_mut() {
        if duration.remain_ticks > 0 {
            duration.remain_ticks -= 1;
        }

        if duration.remain_ticks == 0 {
            // Cleanup Attributes
            if let Ok(mut attr_set) = attr_query.get_mut(effect.target) {
                attr_set.remove_modifiers(entity);
            }
            // Cleanup Tags
            if let Ok(mut tag_container) = tag_query.get_mut(effect.target) {
                tag_container
                    .remove_tags(effect.spec.get_def_tags().get_granted_tags(), &tag_manager);
            }
            // Despawn
            commands.entity(entity).despawn();
        }
    }
}

pub fn tick_effect_period_system(
    mut query: Query<(&mut ActiveEffectPeriod, &ActiveGameplayEffect)>,
    mut attr_query: Query<&mut AttributeSet>,
) {
    for (mut period, effect) in query.iter_mut() {
        period.current_tick += 1;
        if period.current_tick >= period.period_ticks {
            period.current_tick = 0;
            if let Ok(mut attr_set) = attr_query.get_mut(effect.target) {
                for mod_spec in effect.spec.get_modifier_specs() {
                    attr_set.apply_instant_modifier(mod_spec);
                }
            }
        }
    }
}
