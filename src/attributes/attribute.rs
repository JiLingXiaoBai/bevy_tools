use super::attribute_aggregator::{Aggregator, default_executor};
#[derive(Debug, Clone)]
pub struct Attribute {
    base: f64,
    current: f64,
    aggregator: Aggregator,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base: 0.0,
            current: 0.0,
            aggregator: Aggregator::new(default_executor),
        }
    }
}

impl Attribute {
    pub fn init(&mut self, base_value: f64, executor: Option<fn(&Aggregator, f64) -> f64>) {
        self.base = base_value;
        if let Some(executor) = executor {
            self.aggregator.set_executor(executor);
        }
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        self.current = self.aggregator.evaluate(self.base);
    }
}
