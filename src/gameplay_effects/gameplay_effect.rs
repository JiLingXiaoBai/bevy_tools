use super::gameplay_effect_context::EffectContext;
use super::gameplay_effect_spec::{EffectDurationSpec, EffectPeriodSpec, GameplayEffectSpec};
use crate::gameplay_tags::GameplayTag;
use crate::modifiers::{Modifier, ModifierMagnitude};
use std::sync::Arc;

pub enum EffectDuration {
    Instant,
    Duration(ModifierMagnitude),
    Infinite,
}

impl EffectDuration {
    pub fn make_spec(&self, context: &EffectContext) -> EffectDurationSpec {
        match self {
            EffectDuration::Instant => EffectDurationSpec::Instant,
            EffectDuration::Duration(mm) => EffectDurationSpec::Duration(match mm {
                ModifierMagnitude::Flat(f) => *f,
                ModifierMagnitude::Calculated(mmc) => mmc.calculate(context),
            }),
            EffectDuration::Infinite => EffectDurationSpec::Infinite,
        }
    }
}

pub struct EffectPeriod {
    period: ModifierMagnitude,
    execute_on_applied: bool,
}

impl EffectPeriod {
    pub fn make_spec(&self, context: &EffectContext) -> EffectPeriodSpec {
        let final_value = match &self.period {
            ModifierMagnitude::Flat(f) => *f,
            ModifierMagnitude::Calculated(mmc) => mmc.calculate(context),
        };
        EffectPeriodSpec::new(final_value, self.execute_on_applied)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StackingType {
    None,
    Aggregate,
    Override,
}

pub struct EffectTags {
    asset_tags: Vec<GameplayTag>,
    granted_tags: Vec<GameplayTag>,
    required_tags: Vec<GameplayTag>,
    blocked_tags: Vec<GameplayTag>,
    remove_effects_with_tags: Vec<GameplayTag>,
}

impl EffectTags {
    pub fn get_asset_tags(&self) -> &[GameplayTag] {
        &self.asset_tags
    }

    pub fn get_granted_tags(&self) -> &[GameplayTag] {
        &self.granted_tags
    }

    pub fn get_required_tags(&self) -> &[GameplayTag] {
        &self.required_tags
    }

    pub fn get_blocked_tags(&self) -> &[GameplayTag] {
        &self.blocked_tags
    }

    pub fn get_remove_effects_with_tags(&self) -> &[GameplayTag] {
        &self.remove_effects_with_tags
    }
}

// stored as a Resource
pub struct GameplayEffect {
    modifiers: Vec<Modifier>,
    duration: EffectDuration,
    period: Option<EffectPeriod>,
    _probability_to_apply: f64,
    stacking_type: StackingType,
    stacking_limit: u32,
    tags: EffectTags,
}

impl GameplayEffect {
    pub fn make_spec(self: &Arc<Self>, context: EffectContext) -> GameplayEffectSpec {
        GameplayEffectSpec::new(
            self.clone(),
            self.modifiers
                .iter()
                .map(|m| m.make_spec(&context))
                .collect(),
            self.duration.make_spec(&context),
            self.period.as_ref().map(|p| p.make_spec(&context)),
            self.stacking_type,
            self.stacking_limit,
            context.level,
        )
    }

    pub fn get_tags(&self) -> &EffectTags {
        &self.tags
    }
}
