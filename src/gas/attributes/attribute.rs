use super::attribute_aggregator::Aggregator;
use super::attribute_id_manager::AttributeId;
use super::attribute_snapshot::AttributeSnapshot;
use crate::gameplay_effects::ActiveEffectHandle;
use crate::modifiers::{ModifierOperation, ModifierSpec};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttributeClampBound {
    Static(f64),
    Attribute(AttributeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AttributeClamp {
    #[default]
    None,
    Range {
        min: Option<AttributeClampBound>,
        max: Option<AttributeClampBound>,
    },
}

#[derive(Debug, Clone)]
pub struct Attribute {
    base: f64,
    current: f64,
    aggregator: Aggregator,
    dirty: bool,
    clamp: AttributeClamp,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base: 0.0,
            current: 0.0,
            aggregator: Aggregator::default(),
            dirty: true,
            clamp: AttributeClamp::None,
        }
    }
}

impl Attribute {
    pub fn init(&mut self, base_value: f64, executor: Option<fn(&Aggregator, f64) -> f64>) {
        self.init_with_clamp(base_value, executor, AttributeClamp::None);
    }

    pub fn init_with_clamp(
        &mut self,
        base_value: f64,
        executor: Option<fn(&Aggregator, f64) -> f64>,
        clamp: AttributeClamp,
    ) {
        self.base = base_value;
        self.clamp = clamp;
        self.clamp_base_static();
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

    pub fn get_base_value(&self) -> f64 {
        self.base
    }

    pub fn get_clamp(&self) -> AttributeClamp {
        self.clamp
    }

    pub fn set_clamp(&mut self, clamp: AttributeClamp) {
        self.clamp = clamp;
        self.clamp_base_static();
        self.make_dirty();
    }

    pub fn clamp_current(&mut self, min: Option<f64>, max: Option<f64>) {
        let mut value = self.current;
        if let Some(min) = min {
            value = value.max(min);
        }
        if let Some(max) = max {
            value = value.min(max);
        }
        self.current = value;
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
    }

    pub fn modify_base_value(&mut self, spec: &ModifierSpec) {
        match spec.get_operation() {
            ModifierOperation::Add => self.base += spec.get_value(),
            ModifierOperation::PercentAdd => self.base *= 1.0 + spec.get_value(),
            ModifierOperation::Multiply => self.base *= spec.get_value(),
            ModifierOperation::Override => self.base = spec.get_value(),
        }
        self.clamp_base_static();
        self.make_dirty();
    }

    pub fn reset_aggregator(&mut self) {
        self.aggregator.reset();
        self.make_dirty();
    }

    pub fn make_snapshot(&self) -> AttributeSnapshot {
        AttributeSnapshot::new(self.base, self.current)
    }

    fn clamp_base_static(&mut self) {
        let AttributeClamp::Range { min, max } = self.clamp else {
            return;
        };

        if let Some(AttributeClampBound::Static(min)) = min {
            self.base = self.base.max(min);
        }
        if let Some(AttributeClampBound::Static(max)) = max {
            self.base = self.base.min(max);
        }
    }
}
