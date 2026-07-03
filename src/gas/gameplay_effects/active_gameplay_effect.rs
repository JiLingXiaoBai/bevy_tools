use super::gameplay_effect::{GameplayEffect, StackingType};
use super::gameplay_effect_spec::{EffectDurationTicksSpec, GameplayEffectSpec};
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
    stack_count: u32,
    inhibited: bool,
}

impl ActiveGameplayEffect {
    pub fn new(spec: GameplayEffectSpec, source: Entity, target: Entity) -> Self {
        Self {
            spec,
            source,
            target,
            stack_count: 1,
            inhibited: false,
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

    pub fn get_stack_count(&self) -> u32 {
        self.stack_count
    }

    pub fn set_stack_count(&mut self, stack_count: u32) {
        self.stack_count = stack_count.max(1);
    }

    pub fn is_inhibited(&self) -> bool {
        self.inhibited
    }

    pub fn set_inhibited(&mut self, inhibited: bool) {
        self.inhibited = inhibited;
    }
}

#[derive(Component)]
pub struct ActiveEffectDurationTicks {
    remain_ticks: u32,
}

#[derive(Component)]
pub struct ActiveEffectPeriodTicks {
    period_ticks: u32,
    current_tick: u32,
}

pub struct GameplayEffectApplicationPlan {
    source: Entity,
    target: Entity,
    spec: GameplayEffectSpec,
    removed_effects: Vec<ActiveEffectHandle>,
    kind: GameplayEffectApplicationKind,
}

enum GameplayEffectApplicationKind {
    Instant,
    StackExisting {
        handle: ActiveEffectHandle,
        new_stack_count: u32,
    },
    CreateActive,
}

pub fn prepare_gameplay_effect(
    source: Entity,
    target: Entity,
    effect_def: &Arc<GameplayEffect>,
    params: &mut AbilitySystemParams,
    level: u32,
) -> Option<GameplayEffectApplicationPlan> {
    let probability = effect_def.get_probability_to_apply();
    if probability < 1.0 && !params.random_gen.random_bool(probability) {
        return None;
    }

    let incoming_tags = effect_def.get_tags();
    if !passes_application_requirements(source, target, incoming_tags, params) {
        return None;
    }

    if is_blocked_by_application_immunity(source, target, incoming_tags, params) {
        return None;
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
    if matches!(duration_spec, EffectDurationTicksSpec::DurationTicks(0)) {
        return None;
    }

    let needs_attribute_set = duration_spec.is_instant()
        || spec.get_period_spec().is_none()
        || spec
            .get_period_spec()
            .as_ref()
            .is_some_and(|period| period.get_execute_on_applied() || period.get_period_ticks() > 0);
    if needs_attribute_set && params.attr_set_query.get(target).is_err() {
        return None;
    }

    let removed_effects = collect_active_effects_with_tags_for_params(
        target,
        incoming_tags.get_remove_effects_with_tags(),
        &mut params.active_effect_query,
        &params.tag_manager,
    );

    if let Some((handle, stack_count)) = find_stackable_active_effect(
        source,
        target,
        &spec,
        &mut params.active_effect_query,
        &removed_effects,
    ) {
        let limit = spec.get_stacking_limit();
        if limit != 0 && stack_count >= limit {
            return None;
        }

        return Some(GameplayEffectApplicationPlan {
            source,
            target,
            spec,
            removed_effects,
            kind: GameplayEffectApplicationKind::StackExisting {
                handle,
                new_stack_count: stack_count.saturating_add(1),
            },
        });
    }

    let kind = if duration_spec.is_instant() {
        GameplayEffectApplicationKind::Instant
    } else {
        GameplayEffectApplicationKind::CreateActive
    };

    Some(GameplayEffectApplicationPlan {
        source,
        target,
        spec,
        removed_effects,
        kind,
    })
}

pub fn execute_gameplay_effect_plan(
    plan: GameplayEffectApplicationPlan,
    params: &mut AbilitySystemParams,
) -> bool {
    remove_collected_active_effects_for_params(
        &plan.removed_effects,
        &mut params.active_effect_query,
        &mut params.commands,
        &mut params.attr_set_query,
        &mut params.tag_container_query,
        &params.tag_manager,
    );

    match plan.kind {
        GameplayEffectApplicationKind::Instant => execute_instant_effect(&plan, params),
        GameplayEffectApplicationKind::StackExisting {
            handle,
            new_stack_count,
        } => execute_stack_existing_effect(&plan, handle, new_stack_count, params),
        GameplayEffectApplicationKind::CreateActive => execute_new_active_effect(&plan, params),
    }
}

pub fn apply_gameplay_effect(
    source: Entity,
    target: Entity,
    effect_def: &Arc<GameplayEffect>,
    params: &mut AbilitySystemParams,
    level: u32,
) -> bool {
    let Some(plan) = prepare_gameplay_effect(source, target, effect_def, params, level) else {
        return false;
    };

    execute_gameplay_effect_plan(plan, params)
}

fn execute_instant_effect(
    plan: &GameplayEffectApplicationPlan,
    params: &mut AbilitySystemParams,
) -> bool {
    let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(plan.target) else {
        return false;
    };
    apply_instant_modifiers(&mut target_attrs_mut, &plan.spec, 1);
    true
}

fn execute_stack_existing_effect(
    plan: &GameplayEffectApplicationPlan,
    handle: ActiveEffectHandle,
    new_stack_count: u32,
    params: &mut AbilitySystemParams,
) -> bool {
    let Ok((_, mut active_effect, duration, period)) = params.active_effect_query.get_mut(handle)
    else {
        return execute_new_active_effect(plan, params);
    };

    active_effect.set_stack_count(new_stack_count);
    let existing_target = active_effect.get_target();
    let existing_spec = active_effect.get_spec().clone();

    if let (EffectDurationTicksSpec::DurationTicks(duration_ticks), Some(mut duration)) =
        (plan.spec.get_duration_spec(), duration)
    {
        duration.remain_ticks = *duration_ticks;
    }

    if let Some(mut period) = period {
        period.current_tick = 0;
    }

    if !active_effect.is_inhibited() && existing_spec.get_period_spec().is_none() {
        let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(existing_target) else {
            return false;
        };
        target_attrs_mut.remove_modifiers(handle);
        apply_duration_modifiers(
            &mut target_attrs_mut,
            &existing_spec,
            handle,
            new_stack_count,
        );
    }

    true
}

fn execute_new_active_effect(
    plan: &GameplayEffectApplicationPlan,
    params: &mut AbilitySystemParams,
) -> bool {
    let mut entity_cmds = params.commands.spawn(ActiveGameplayEffect::new(
        plan.spec.clone(),
        plan.source,
        plan.target,
    ));

    let effect_entity = entity_cmds.id();

    if let EffectDurationTicksSpec::DurationTicks(duration) = plan.spec.get_duration_spec() {
        entity_cmds.insert(ActiveEffectDurationTicks {
            remain_ticks: *duration,
        });
    }

    if let Some(period_spec) = plan.spec.get_period_spec() {
        let period_ticks = period_spec.get_period_ticks();
        let execute_on_application = period_spec.get_execute_on_applied();
        if execute_on_application {
            let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(plan.target) else {
                params.commands.entity(effect_entity).despawn();
                return false;
            };
            apply_instant_modifiers(&mut target_attrs_mut, &plan.spec, 1);
        }
        if period_ticks > 0 {
            entity_cmds.insert(ActiveEffectPeriodTicks {
                period_ticks,
                current_tick: 0,
            });
        }
    } else {
        let Ok(mut target_attrs_mut) = params.attr_set_query.get_mut(plan.target) else {
            params.commands.entity(effect_entity).despawn();
            return false;
        };
        apply_duration_modifiers(&mut target_attrs_mut, &plan.spec, effect_entity, 1);
    }

    entity_cmds.set_parent_in_place(plan.target);

    let Ok(mut target_tags) = params.tag_container_query.get_mut(plan.target) else {
        params.commands.entity(effect_entity).despawn();
        return false;
    };
    target_tags.add_tags(
        plan.spec.get_def_tags().get_granted_tags(),
        &params.tag_manager,
    );

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
    mut query: Query<(
        Entity,
        &mut ActiveEffectDurationTicks,
        &ActiveGameplayEffect,
    )>,
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

pub fn update_active_effect_tag_requirements_system(
    mut commands: Commands,
    mut active_effect_query: Query<(Entity, &mut ActiveGameplayEffect)>,
    mut attr_query: Query<&mut AttributeSet>,
    mut tag_query: Query<&mut GameplayTagContainer>,
    tag_manager: Res<GameplayTagManager>,
) {
    for (handle, mut effect) in active_effect_query.iter_mut() {
        if should_remove_active_effect(&effect, &tag_query) {
            cleanup_active_gameplay_effect(
                &mut commands,
                handle,
                &effect,
                &mut attr_query,
                &mut tag_query,
                &tag_manager,
            );
            continue;
        }

        let ongoing_passes = passes_ongoing_requirements(&effect, &tag_query);
        match (ongoing_passes, effect.is_inhibited()) {
            (false, false) => {
                inhibit_active_effect(
                    handle,
                    &mut effect,
                    &mut attr_query,
                    &mut tag_query,
                    &tag_manager,
                );
            }
            (true, true) => {
                uninhibit_active_effect(
                    handle,
                    &mut effect,
                    &mut attr_query,
                    &mut tag_query,
                    &tag_manager,
                );
            }
            _ => {}
        }
    }
}

pub fn tick_effect_period_system(
    mut query: Query<(&mut ActiveEffectPeriodTicks, &ActiveGameplayEffect)>,
    mut attr_query: Query<&mut AttributeSet>,
) {
    for (mut period, effect) in query.iter_mut() {
        if effect.is_inhibited() {
            continue;
        }

        period.current_tick += 1;
        if period.current_tick >= period.period_ticks {
            period.current_tick = 0;
            if let Ok(mut attr_set) = attr_query.get_mut(effect.get_target()) {
                apply_instant_modifiers(&mut attr_set, effect.get_spec(), effect.get_stack_count());
            }
        }
    }
}

fn find_stackable_active_effect(
    source: Entity,
    target: Entity,
    spec: &GameplayEffectSpec,
    active_effect_query: &mut Query<(
        Entity,
        &mut ActiveGameplayEffect,
        Option<&mut ActiveEffectDurationTicks>,
        Option<&mut ActiveEffectPeriodTicks>,
    )>,
    ignored_handles: &[ActiveEffectHandle],
) -> Option<(ActiveEffectHandle, u32)> {
    let stacking_type = spec.get_stacking_type();
    if matches!(stacking_type, StackingType::None) {
        return None;
    }

    active_effect_query
        .iter_mut()
        .find_map(|(handle, effect, _, _)| {
            (!ignored_handles.contains(&handle)
                && effect.get_target() == target
                && spec.is_same_def(effect.get_spec())
                && match stacking_type {
                    StackingType::None => false,
                    StackingType::AggregateBySource => effect.get_source() == source,
                    StackingType::AggregateByTarget => true,
                })
            .then_some((handle, effect.get_stack_count()))
        })
}

fn passes_application_requirements(
    source: Entity,
    target: Entity,
    incoming_tags: &crate::gameplay_effects::EffectTags,
    params: &AbilitySystemParams,
) -> bool {
    let source_tags = params.tag_container_query.get(source).ok();
    let target_tags = params.tag_container_query.get(target).ok();

    incoming_tags
        .get_source_application_tags()
        .passes(source_tags)
        && incoming_tags
            .get_target_application_tags()
            .passes(target_tags)
}

fn is_blocked_by_application_immunity(
    source: Entity,
    target: Entity,
    incoming_tags: &crate::gameplay_effects::EffectTags,
    params: &mut AbilitySystemParams,
) -> bool {
    let source_tags = params.tag_container_query.get(source).ok();
    let incoming_asset_tags = incoming_tags.get_asset_tags();

    params
        .active_effect_query
        .iter_mut()
        .filter(|(_, active_effect, _, _)| {
            active_effect.get_target() == target && !active_effect.is_inhibited()
        })
        .any(|(_, active_effect, _, _)| {
            active_effect
                .get_spec()
                .get_def_tags()
                .get_granted_application_immunity()
                .iter()
                .any(|immunity| {
                    immunity.matches(source_tags, incoming_asset_tags, &params.tag_manager)
                })
        })
}

fn should_remove_active_effect(
    effect: &ActiveGameplayEffect,
    tag_query: &Query<&mut GameplayTagContainer>,
) -> bool {
    let source_tags = tag_query.get(effect.get_source()).ok();
    let target_tags = tag_query.get(effect.get_target()).ok();
    let effect_tags = effect.get_spec().get_def_tags();

    removal_requirement_matches(effect_tags.get_source_removal_tags(), source_tags)
        || removal_requirement_matches(effect_tags.get_target_removal_tags(), target_tags)
}

fn removal_requirement_matches(
    requirements: &crate::gameplay_effects::TagRequirements,
    tags: Option<&GameplayTagContainer>,
) -> bool {
    !requirements.is_empty() && requirements.passes(tags)
}

fn passes_ongoing_requirements(
    effect: &ActiveGameplayEffect,
    tag_query: &Query<&mut GameplayTagContainer>,
) -> bool {
    let source_tags = tag_query.get(effect.get_source()).ok();
    let target_tags = tag_query.get(effect.get_target()).ok();
    let effect_tags = effect.get_spec().get_def_tags();

    effect_tags.get_source_ongoing_tags().passes(source_tags)
        && effect_tags.get_target_ongoing_tags().passes(target_tags)
}

fn inhibit_active_effect(
    handle: ActiveEffectHandle,
    effect: &mut ActiveGameplayEffect,
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

    effect.set_inhibited(true);
}

fn uninhibit_active_effect(
    handle: ActiveEffectHandle,
    effect: &mut ActiveGameplayEffect,
    attr_query: &mut Query<&mut AttributeSet>,
    tag_query: &mut Query<&mut GameplayTagContainer>,
    tag_manager: &Res<GameplayTagManager>,
) {
    if effect.get_spec().get_period_spec().is_none()
        && let Ok(mut attr_set) = attr_query.get_mut(effect.get_target())
    {
        apply_duration_modifiers(
            &mut attr_set,
            effect.get_spec(),
            handle,
            effect.get_stack_count(),
        );
    }

    if let Ok(mut tag_container) = tag_query.get_mut(effect.get_target()) {
        tag_container.add_tags(
            effect.get_spec().get_def_tags().get_granted_tags(),
            tag_manager,
        );
    }

    effect.set_inhibited(false);
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

fn collect_active_effects_with_tags_for_params(
    target: Entity,
    tags: &[GameplayTag],
    active_effect_query: &mut Query<(
        Entity,
        &mut ActiveGameplayEffect,
        Option<&mut ActiveEffectDurationTicks>,
        Option<&mut ActiveEffectPeriodTicks>,
    )>,
    tag_manager: &Res<GameplayTagManager>,
) -> Vec<ActiveEffectHandle> {
    if tags.is_empty() {
        return Vec::new();
    }

    active_effect_query
        .iter_mut()
        .filter_map(|(handle, effect, _, _)| {
            (effect.get_target() == target
                && active_effect_has_any_tags(&effect, tags, tag_manager))
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

fn remove_collected_active_effects_for_params(
    handles: &[ActiveEffectHandle],
    active_effect_query: &mut Query<(
        Entity,
        &mut ActiveGameplayEffect,
        Option<&mut ActiveEffectDurationTicks>,
        Option<&mut ActiveEffectPeriodTicks>,
    )>,
    commands: &mut Commands,
    attr_query: &mut Query<&mut AttributeSet>,
    tag_query: &mut Query<&mut GameplayTagContainer>,
    tag_manager: &Res<GameplayTagManager>,
) {
    for &handle in handles {
        let Ok((_, effect, _, _)) = active_effect_query.get_mut(handle) else {
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

fn apply_duration_modifiers(
    attr_set: &mut AttributeSet,
    spec: &GameplayEffectSpec,
    handle: ActiveEffectHandle,
    stack_count: u32,
) {
    for mod_spec in spec.get_modifier_specs() {
        let stacked_spec = mod_spec.scaled_by_stack(stack_count);
        attr_set.apply_duration_modifier(&stacked_spec, handle);
    }
}

fn apply_instant_modifiers(
    attr_set: &mut AttributeSet,
    spec: &GameplayEffectSpec,
    stack_count: u32,
) {
    for mod_spec in spec.get_modifier_specs() {
        let stacked_spec = mod_spec.scaled_by_stack(stack_count);
        attr_set.apply_instant_modifier(&stacked_spec);
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
