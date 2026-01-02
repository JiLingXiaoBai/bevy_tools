use super::gameplay_effect::GameplayEffect;
use super::gameplay_effect_context::{EffectContext, EffectContextEntityType};
use super::gameplay_effect_spec::{EffectDurationSpec, GameplayEffectSpec};
use crate::gameplay_tags::GameplayTagManager;
use crate::randoms::Random;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveEffectHandle(u64);

pub struct ActiveGameplayEffect {
    _handle: ActiveEffectHandle,
    spec: GameplayEffectSpec,
    start_time: f64,
    _last_period_tick_time: Option<f64>,
    _is_inhibited: bool,
}

impl ActiveGameplayEffect {
    pub fn new(handle: ActiveEffectHandle, spec: GameplayEffectSpec, start_time: f64) -> Self {
        Self {
            _handle: handle,
            spec,
            start_time,
            _last_period_tick_time: None,
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
}

#[derive(Resource, Default)]
pub struct ActiveEffectHandleGenerator(pub u64);

impl ActiveEffectHandleGenerator {
    pub fn generate(&mut self) -> ActiveEffectHandle {
        self.0 += 1;
        ActiveEffectHandle(self.0)
    }
}

#[derive(Component, Default)]
pub struct ActiveEffects {
    pub list: Vec<ActiveGameplayEffect>,
}

pub fn apply_gameplay_effect(
    effect_def: &Arc<GameplayEffect>,
    context: &mut EffectContext,
    tag_manager: &Res<GameplayTagManager>,
    handle_gen: &mut ResMut<ActiveEffectHandleGenerator>,
    random_gen: &mut ResMut<Random>,
    time: &Res<Time>,
) {
    let probability = effect_def.get_probability_to_apply();
    if probability < 1.0 && !random_gen.random_bool(probability) {
        return;
    }

    if let Some(target_tags) = context.get_tag_container_mut(EffectContextEntityType::Target) {
        let tags = effect_def.get_tags();
        if !target_tags.has_all(tags.get_required_tags())
            || target_tags.has_any(tags.get_blocked_tags())
        {
            return;
        }
    }

    let spec = effect_def.make_spec(context);
    let duration_spec = spec.get_duration_spec();

    let mut target_attrs = context
        .get_attr_set_mut(EffectContextEntityType::Target)
        .expect("Target has no attribute set");

    if duration_spec.is_instant() {
        for mod_spec in spec.get_modifier_specs() {
            target_attrs.apply_instant_modifier(mod_spec);
        }
        return;
    }

    // TODO: Stacking Logic

    let handle = handle_gen.generate();
    let start_time = time.elapsed_secs_f64();

    let active_effect = ActiveGameplayEffect::new(handle, spec.clone(), start_time);
    for mod_spec in spec.get_modifier_specs() {
        target_attrs.apply_duration_modifier(mod_spec, handle);
    }

    let mut target_tag_container = context
        .get_tag_container_mut(EffectContextEntityType::Target)
        .expect("Target has no TagContainer");
    target_tag_container.add_tags(spec.get_granted_tags(), tag_manager);

    let mut target_active_effects = context
        .get_active_effects_mut(EffectContextEntityType::Target)
        .expect("Target has no ActiveEffects");

    target_active_effects.list.push(active_effect);
}
