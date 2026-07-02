use super::gameplay_effect::{GameplayEffect, StackingType};
use super::gameplay_effect_spec::{EffectDurationSpec, GameplayEffectSpec};
use crate::ability_system::AbilitySystemParams;
use crate::attributes::AttributeSet;
use crate::gameplay_tags::{GameplayTag, GameplayTagContainer, GameplayTagManager};
use bevy::prelude::*;
use std::sync::Arc;

pub type ActiveEffectHandle = Entity;

#[derive(Component, Clone)]
pub struct ActiveGameplayEffect {
    spec: GameplayEffectSpec,
    source: Entity,
    target: Entity,
}

impl ActiveGameplayEffect {
    pub fn new(spec: GameplayEffectSpec, source: Entity, target: Entity) -> Self {
        Self {
            spec,
            source,
            target,
        }
    }

    pub fn get_spec(&self) -> &GameplayEffectSpec {
        &self.spec
    }

    pub fn get_source(&self) -> Entity {
        self.source
    }

    pub fn get_target(&self) -> Entity {
        self.target
    }
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
) -> bool {
    let probability = effect_def.get_probability_to_apply();
    if probability < 1.0 && !params.random_gen.random_bool(probability) {
        return false;
    }

    let tags = effect_def.get_tags();

    let Ok(target_tags) = params.tag_container_query.get(target) else {
        return false;
    };
    if !target_tags.has_all(tags.get_required_tags())
        || target_tags.has_any(tags.get_blocked_tags())
    {
        return false;
    }

    let spec = {
        let context = crate::gameplay_effects::EffectContext {
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
    if matches!(duration_spec, EffectDurationSpec::Duration(0)) {
        return false;
    }

    let needs_attribute_set = duration_spec.is_instant()
        || spec.get_period_spec().is_none()
        || spec
            .get_period_spec()
            .as_ref()
            .is_some_and(|period| period.get_execute_on_applied() || period.get_period() > 0);
    if needs_attribute_set && params.attr_set_query.get(target).is_err() {
        return false;
    }

    let ignored_handles = collect_active_effects_with_tags(
        target,
        tags.get_remove_effects_with_tags(),
        &params.active_effect_query,
        &params.tag_manager,
    );

    if !can_apply_stacking_policy(
        source,
        target,
        &spec,
        &params.active_effect_query,
        &ignored_handles,
    ) {
        return false;
    }

    remove_collected_active_effects(
        &ignored_handles,
        &params.active_effect_query,
        &mut params.commands,
        &mut params.attr_set_query,
        &mut params.tag_container_query,
        &params.tag_manager,
    );

    if duration_spec.is_instant() {
        let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(target) else {
            return false;
        };
        for mod_spec in spec.get_modifier_specs() {
            target_attrs_mut.apply_instant_modifier(mod_spec);
        }
        return true;
    }

    let mut entity_cmds =
        params
            .commands
            .spawn(ActiveGameplayEffect::new(spec.clone(), source, target));

    let effect_entity = entity_cmds.id();

    if let EffectDurationSpec::Duration(duration) = duration_spec {
        entity_cmds.insert(ActiveEffectDuration {
            remain_ticks: *duration,
        });
    }

    if let Some(period_spec) = spec.get_period_spec() {
        let period = period_spec.get_period();
        let execute_on_application = period_spec.get_execute_on_applied();
        if execute_on_application {
            let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(target) else {
                params.commands.entity(effect_entity).despawn();
                return false;
            };
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
    } else {
        let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(target) else {
            params.commands.entity(effect_entity).despawn();
            return false;
        };
        for mod_spec in spec.get_modifier_specs() {
            target_attrs_mut.apply_duration_modifier(mod_spec, effect_entity);
        }
    }

    entity_cmds.set_parent_in_place(target);

    let Ok(mut target_tags) = params.tag_container_query.get_mut(target) else {
        params.commands.entity(effect_entity).despawn();
        return false;
    };
    target_tags.add_tags(tags.get_granted_tags(), &params.tag_manager);

    true
}
pub fn remove_active_effect(
    handle: ActiveEffectHandle,
    commands: &mut Commands,
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
    attr_query: &mut Query<&mut AttributeSet>,
    tag_query: &mut Query<&mut GameplayTagContainer>,
    tag_manager: &Res<GameplayTagManager>,
) -> bool {
    let Ok((_, effect)) = active_effect_query.get(handle) else {
        return false;
    };
    let effect = effect.clone();
    cleanup_active_gameplay_effect(
        commands,
        handle,
        &effect,
        attr_query,
        tag_query,
        tag_manager,
    );
    true
}

pub fn remove_active_effects_with_tags(
    target: Entity,
    tags: &[GameplayTag],
    commands: &mut Commands,
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
    attr_query: &mut Query<&mut AttributeSet>,
    tag_query: &mut Query<&mut GameplayTagContainer>,
    tag_manager: &Res<GameplayTagManager>,
) -> usize {
    let handles = collect_active_effects_with_tags(target, tags, active_effect_query, tag_manager);
    let removed_count = handles.len();
    remove_collected_active_effects(
        &handles,
        active_effect_query,
        commands,
        attr_query,
        tag_query,
        tag_manager,
    );
    removed_count
}

pub fn get_active_effects_on_target(
    target: Entity,
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
) -> Vec<ActiveEffectHandle> {
    active_effect_query
        .iter()
        .filter_map(|(handle, effect)| (effect.get_target() == target).then_some(handle))
        .collect()
}

pub fn has_active_effect_with_tags(
    target: Entity,
    tags: &[GameplayTag],
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
    tag_manager: &Res<GameplayTagManager>,
) -> bool {
    if tags.is_empty() {
        return false;
    }

    active_effect_query.iter().any(|(_, effect)| {
        effect.get_target() == target && active_effect_has_any_tags(effect, tags, tag_manager)
    })
}

pub fn cleanup_active_gameplay_effect(
    commands: &mut Commands,
    handle: ActiveEffectHandle,
    effect: &ActiveGameplayEffect,
    attr_query: &mut Query<&mut AttributeSet>,
    tag_query: &mut Query<&mut GameplayTagContainer>,
    tag_manager: &Res<GameplayTagManager>,
) {
    if let Ok(mut attr_set) = attr_query.get_mut(effect.get_target()) {
        attr_set.remove_modifiers(handle);
    }

    if let Ok(mut tag_container) = tag_query.get_mut(effect.get_target()) {
        tag_container.remove_tags(
            effect.get_spec().get_def_tags().get_granted_tags(),
            tag_manager,
        );
    }

    commands.entity(handle).despawn();
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
            cleanup_active_gameplay_effect(
                &mut commands,
                entity,
                effect,
                &mut attr_query,
                &mut tag_query,
                &tag_manager,
            );
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
            if let Ok(mut attr_set) = attr_query.get_mut(effect.get_target()) {
                for mod_spec in effect.get_spec().get_modifier_specs() {
                    attr_set.apply_instant_modifier(mod_spec);
                }
            }
        }
    }
}

fn can_apply_stacking_policy(
    source: Entity,
    target: Entity,
    spec: &GameplayEffectSpec,
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
    ignored_handles: &[ActiveEffectHandle],
) -> bool {
    let stacking_type = spec.get_stacking_type();
    if matches!(stacking_type, StackingType::None) {
        return true;
    }

    let matching_count = active_effect_query
        .iter()
        .filter(|(handle, effect)| {
            !ignored_handles.contains(handle)
                && effect.get_target() == target
                && spec.is_same_def(effect.get_spec())
                && match stacking_type {
                    StackingType::None => false,
                    StackingType::AggregateBySource => effect.get_source() == source,
                    StackingType::AggregateByTarget => true,
                }
        })
        .count() as u32;

    let limit = spec.get_stacking_limit();
    limit == 0 || matching_count < limit
}

fn collect_active_effects_with_tags(
    target: Entity,
    tags: &[GameplayTag],
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
    tag_manager: &Res<GameplayTagManager>,
) -> Vec<ActiveEffectHandle> {
    if tags.is_empty() {
        return Vec::new();
    }

    active_effect_query
        .iter()
        .filter_map(|(handle, effect)| {
            (effect.get_target() == target && active_effect_has_any_tags(effect, tags, tag_manager))
                .then_some(handle)
        })
        .collect()
}

fn remove_collected_active_effects(
    handles: &[ActiveEffectHandle],
    active_effect_query: &Query<(Entity, &ActiveGameplayEffect)>,
    commands: &mut Commands,
    attr_query: &mut Query<&mut AttributeSet>,
    tag_query: &mut Query<&mut GameplayTagContainer>,
    tag_manager: &Res<GameplayTagManager>,
) {
    for &handle in handles {
        let Ok((_, effect)) = active_effect_query.get(handle) else {
            continue;
        };
        let effect = effect.clone();
        cleanup_active_gameplay_effect(
            commands,
            handle,
            &effect,
            attr_query,
            tag_query,
            tag_manager,
        );
    }
}

fn active_effect_has_any_tags(
    effect: &ActiveGameplayEffect,
    tags: &[GameplayTag],
    tag_manager: &Res<GameplayTagManager>,
) -> bool {
    let mut effect_tags = GameplayTagContainer::default();
    effect_tags.add_tags(
        effect.get_spec().get_def_tags().get_asset_tags(),
        tag_manager,
    );
    effect_tags.has_any(tags)
}
