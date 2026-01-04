use super::gameplay_effect::GameplayEffect;
use super::gameplay_effect_context::EffectContext;
use super::gameplay_effect_spec::{EffectDurationSpec, GameplayEffectSpec};
use crate::attributes::AttributeSet;
use crate::gameplay_tags::GameplayTagManager;
use crate::randoms::Random;
use crate::{AbilitySystemComponent, GameplayTagContainer};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveEffectHandle(u64);

pub struct ActiveGameplayEffect {
    handle: ActiveEffectHandle,
    spec: GameplayEffectSpec,
    start_time: f64,
    last_period_tick_time: Option<f64>,
    _is_inhibited: bool,
}

impl ActiveGameplayEffect {
    pub fn new(
        handle: ActiveEffectHandle,
        spec: GameplayEffectSpec,
        start_time: f64,
        last_period_tick_time: Option<f64>,
    ) -> Self {
        Self {
            handle,
            spec,
            start_time,
            last_period_tick_time,
            _is_inhibited: false,
        }
    }

    pub fn is_expired(&self, current_time: f64) -> bool {
        match *self.spec.get_duration_spec() {
            EffectDurationSpec::Instant => true,
            EffectDurationSpec::Duration(duration) => (current_time - self.start_time) >= duration,
            EffectDurationSpec::Infinite => false,
        }
    }

    pub fn get_time_remaining(&self, current_time: f64) -> Option<f64> {
        match *self.spec.get_duration_spec() {
            EffectDurationSpec::Instant => None,
            EffectDurationSpec::Duration(duration) => {
                Some(duration - (current_time - self.start_time))
            }
            EffectDurationSpec::Infinite => None,
        }
    }

    pub fn get_handle(&self) -> ActiveEffectHandle {
        self.handle
    }

    pub fn set_last_period_tick_time(&mut self, time: f64) {
        self.last_period_tick_time = Some(time);
    }

    pub fn get_last_period_tick_time(&self) -> Option<f64> {
        self.last_period_tick_time
    }

    pub fn can_period_tick(&self, current_time: f64) -> bool {
        if let Some(period_spec) = &self.spec.get_period_spec()
            && let Some(last_period_tick_time) = self.last_period_tick_time
        {
            let period_duration = period_spec.get_period();
            if period_duration > 0.0 {
                let time_since_last_tick = current_time - last_period_tick_time;
                time_since_last_tick >= period_duration
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Resource, Default)]
pub struct ActiveEffectHandleGenerator(pub u64);

impl ActiveEffectHandleGenerator {
    pub fn generate(&mut self) -> ActiveEffectHandle {
        self.0 += 1;
        ActiveEffectHandle(self.0)
    }
}

#[derive(SystemParam)]
pub struct GameplayEffectParams<'w, 's> {
    pub tag_manager: Res<'w, GameplayTagManager>,
    pub handle_gen: ResMut<'w, ActiveEffectHandleGenerator>,
    pub random_gen: ResMut<'w, Random>,
    pub attr_set_query: Query<'w, 's, &'static mut AttributeSet>,
    pub tag_container_query: Query<'w, 's, &'static mut GameplayTagContainer>,
    pub asc_query: Query<'w, 's, &'static mut AbilitySystemComponent>,
    pub time: Res<'w, Time>,
}

pub fn apply_gameplay_effect(
    source: Entity,
    target: Entity,
    effect_def: &Arc<GameplayEffect>,
    params: &mut GameplayEffectParams,
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
            level,
        };

        effect_def.make_spec(&context)
    };
    let duration_spec = spec.get_duration_spec();

    let mut target_attrs = params.attr_set_query.get_mut(target).unwrap();
    if duration_spec.is_instant() {
        for mod_spec in spec.get_modifier_specs() {
            target_attrs.apply_instant_modifier(mod_spec);
        }
        return;
    }

    // TODO: Stacking Logic, Inhibited Logic

    let start_time = params.time.elapsed_secs_f64();
    let handle = params.handle_gen.generate();
    let period_spec = spec.get_period_spec();
    let mut last_period_tick_time = None;
    if period_spec.is_none_or(|period| {
        let period_duration = period.get_period();
        let execute_on_applied = period.get_execute_on_applied();
        if period_duration > 0.0 {
            if execute_on_applied {
                for mod_spec in spec.get_modifier_specs() {
                    target_attrs.apply_instant_modifier(mod_spec);
                }
                last_period_tick_time = Some(start_time);
            }
            return false;
        }
        true
    }) {
        for mod_spec in spec.get_modifier_specs() {
            target_attrs.apply_duration_modifier(mod_spec, handle);
        }
    }

    let mut target_tags = params.tag_container_query.get_mut(target).unwrap();
    target_tags.add_tags(tags.get_granted_tags(), &params.tag_manager);

    let active_effect = ActiveGameplayEffect::new(handle, spec, start_time, last_period_tick_time);

    let mut target_asc = params.asc_query.get_mut(target).unwrap();
    target_asc.add_active_effect(active_effect);
}

pub fn tick_gameplay_effects_system(
    time: Res<Time>,
    mut query: Query<(
        &mut AbilitySystemComponent,
        &mut AttributeSet,
        &mut GameplayTagContainer,
    )>,
    tag_manager: Res<GameplayTagManager>,
) {
    let current_time = time.elapsed_secs_f64();

    for (mut asc, mut attrs, mut tags) in query.iter_mut() {
        let active_effects = asc.get_active_effects();
        let mut remove_handles = Vec::new();

        for effect in active_effects {
            let spec = &effect.spec;
            if effect.is_expired(current_time) {
                attrs.remove_modifiers(effect.get_handle());
                tags.remove_tags(spec.get_def_tags().get_granted_tags(), &tag_manager);
                remove_handles.push(effect.get_handle());
                continue;
            }

            if effect.can_period_tick(current_time) {
                for mod_spec in spec.get_modifier_specs() {
                    attrs.apply_instant_modifier(mod_spec);
                }
            }
        }

        asc.remove_active_effects(&remove_handles);
    }
}
