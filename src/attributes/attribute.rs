use super::attribute_aggregator::Aggregator;
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

    pub fn get_value(&mut self) -> f64 {
        self.recalculate();
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

    pub fn add_additive(&mut self, value: f64) {
        self.aggregator.add_additive(value);
        self.make_dirty();
    }

    pub fn add_multiplicative(&mut self, value: f64) {
        self.aggregator.add_multiplicative(value);
        self.make_dirty();
    }

    pub fn add_percent_additive(&mut self, value: f64) {
        self.aggregator.add_percent_additive(value);
        self.make_dirty();
    }

    pub fn set_override(&mut self, value: f64) {
        self.aggregator.set_override(value);
        self.make_dirty();
    }

    pub fn reset_aggregator(&mut self) {
        self.aggregator.reset();
        self.make_dirty();
    }
}
