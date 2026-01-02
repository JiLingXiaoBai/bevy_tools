use super::{GameplayEffect, StackingType};
use crate::gameplay_tags::GameplayTag;
use crate::modifiers::ModifierSpec;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum EffectDurationSpec {
    Instant,
    Duration(f64),
    Infinite,
}

impl EffectDurationSpec {
    pub fn is_infinite(&self) -> bool {
        matches!(self, EffectDurationSpec::Infinite)
    }

    pub fn is_instant(&self) -> bool {
        matches!(self, EffectDurationSpec::Instant)
    }

    pub fn is_duration(&self) -> bool {
        matches!(self, EffectDurationSpec::Duration(_))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffectPeriodSpec {
    _period: f64,
    _execute_on_applied: bool,
}

impl EffectPeriodSpec {
    pub fn new(period: f64, execute_on_applied: bool) -> Self {
        Self {
            _period: period,
            _execute_on_applied: execute_on_applied,
        }
    }
}

#[derive(Clone)]
pub struct GameplayEffectSpec {
    def: Arc<GameplayEffect>,
    modifier_specs: Vec<ModifierSpec>,
    duration_spec: EffectDurationSpec,
    _period_spec: Option<EffectPeriodSpec>,
    _stacking_type: StackingType,
    _stacking_limit: u32,
    _level: u32,
}

impl GameplayEffectSpec {
    pub fn new(
        def: Arc<GameplayEffect>,
        modifier_specs: Vec<ModifierSpec>,
        duration_spec: EffectDurationSpec,
        period_spec: Option<EffectPeriodSpec>,
        stacking_type: StackingType,
        stacking_limit: u32,
        level: u32,
    ) -> Self {
        Self {
            def,
            modifier_specs,
            duration_spec,
            _period_spec: period_spec,
            _stacking_type: stacking_type,
            _stacking_limit: stacking_limit,
            _level: level,
        }
    }

    pub fn get_asset_tags(&self) -> &[GameplayTag] {
        self.def.get_tags().get_asset_tags()
    }

    pub fn get_granted_tags(&self) -> &[GameplayTag] {
        self.def.get_tags().get_granted_tags()
    }

    pub fn get_required_tags(&self) -> &[GameplayTag] {
        self.def.get_tags().get_required_tags()
    }

    pub fn get_blocked_tags(&self) -> &[GameplayTag] {
        self.def.get_tags().get_blocked_tags()
    }

    pub fn get_remove_effects_with_tags(&self) -> &[GameplayTag] {
        self.def.get_tags().get_remove_effects_with_tags()
    }

    pub fn get_duration_spec(&self) -> &EffectDurationSpec {
        &(self.duration_spec)
    }

    pub fn get_modifier_specs(&self) -> &[ModifierSpec] {
        &self.modifier_specs
    }
}
