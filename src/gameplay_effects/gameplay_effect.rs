use super::EffectContext;
use crate::gameplay_tags::GameplayTag;
use crate::modifiers::{Modifier, ModifierSpec};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum EffectDuration {
    Instant,
    Duration(f32),
    Infinite,
}

impl EffectDuration {
    pub fn is_infinite(&self) -> bool {
        matches!(self, EffectDuration::Infinite)
    }

    pub fn is_instant(&self) -> bool {
        matches!(self, EffectDuration::Instant)
    }

    pub fn is_duration(&self) -> bool {
        matches!(self, EffectDuration::Duration(_))
    }
}

pub struct EffectTags {
    granted_tags: Vec<GameplayTag>,
    required_tags: Vec<GameplayTag>,
    blocked_tags: Vec<GameplayTag>,
}

// stored as a Resource
pub struct GameplayEffect {
    modifiers: Vec<Modifier>,
    duration: EffectDuration,
    tags: EffectTags,
}

impl GameplayEffect {
    pub fn make_spec(self: &Arc<Self>, context: EffectContext) -> GameplayEffectSpec {
        GameplayEffectSpec {
            def: self.clone(),
            _modifier_specs: self
                .modifiers
                .iter()
                .map(|m| m.make_spec(&context))
                .collect(),
            duration: self.duration,
            _level: context.level,
        }
    }
}

#[derive(Clone)]
pub struct GameplayEffectSpec {
    def: Arc<GameplayEffect>,
    _modifier_specs: Vec<ModifierSpec>,
    duration: EffectDuration,
    _level: u32,
}

impl GameplayEffectSpec {
    pub fn get_granted_tags(&self) -> &[GameplayTag] {
        &self.def.tags.granted_tags
    }

    pub fn get_required_tags(&self) -> &[GameplayTag] {
        &self.def.tags.required_tags
    }

    pub fn get_blocked_tags(&self) -> &[GameplayTag] {
        &self.def.tags.blocked_tags
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveEffectHandle(u64);

pub struct ActiveGameplayEffect {
    _handle: ActiveEffectHandle,
    spec: GameplayEffectSpec,
    start_time: f64,
    _is_inhibited: bool,
}

impl ActiveGameplayEffect {
    pub fn is_expired(&self, current_time: f64) -> bool {
        match self.spec.duration {
            EffectDuration::Instant => true,
            EffectDuration::Duration(duration) => {
                (current_time - self.start_time) >= duration as f64
            }
            EffectDuration::Infinite => false,
        }
    }

    pub fn get_time_remaining(&self, current_time: f64) -> Option<f64> {
        match self.spec.duration {
            EffectDuration::Instant => None,
            EffectDuration::Duration(duration) => {
                Some(duration as f64 - (current_time - self.start_time))
            }
            EffectDuration::Infinite => None,
        }
    }
}
