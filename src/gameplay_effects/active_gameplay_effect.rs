use super::gameplay_effect_spec::{EffectDurationSpec, GameplayEffectSpec};

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
