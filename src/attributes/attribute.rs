use super::attribute_aggregator::Aggregator;
use crate::gameplay_effects::ActiveEffectHandle;
use crate::modifiers::ModifierSpec;
#[derive(Debug, Clone)]
pub struct Attribute {
    base: f64,
    current: f64,
    aggregator: Aggregator,
    dirty: bool,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base: 0.0,
            current: 0.0,
            aggregator: Aggregator::default(),
            dirty: true,
        }
    }
}

impl Attribute {
    pub fn init(&mut self, base_value: f64, executor: Option<fn(&Aggregator, f64) -> f64>) {
        self.base = base_value;
        self.set_executor(executor);
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        if self.dirty {
            self.current = self.aggregator.evaluate(self.base);
            self.dirty = false;
        }
    }

    pub fn get_current_value(&self) -> f64 {
        debug_assert!(!self.dirty);
        self.current
    }

    #[inline]
    pub fn make_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn set_executor(&mut self, executor: Option<fn(&Aggregator, f64) -> f64>) {
        self.aggregator.set_executor(executor);
        self.make_dirty();
    }

    pub fn apply_modifier_spec(&mut self, spec: &ModifierSpec, handle: ActiveEffectHandle) {
        self.aggregator.apply_modifier_spec(spec, handle);
        self.make_dirty();
    }

    pub fn remove_modifier_by_handle(&mut self, handle: ActiveEffectHandle) {
        self.aggregator.remove_modifier_by_handle(handle);
        self.make_dirty();
        self.recalculate();
    }

    pub fn reset_aggregator(&mut self) {
        self.aggregator.reset();
        self.make_dirty();
    }
}
