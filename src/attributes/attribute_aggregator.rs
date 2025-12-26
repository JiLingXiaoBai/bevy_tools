pub fn default_executor(aggregator: &Aggregator, base_value: f64) -> f64 {
    if let Some(override_value) = aggregator.override_value {
        return override_value;
    }
    let mut final_value = base_value;
    for &add in &aggregator.additive {
        final_value += add;
    }
    let mut percent_sum = 0.0;
    for &percent in &aggregator.percent_additive {
        percent_sum += percent;
    }
    final_value *= 1.0 + percent_sum;

    for &multiplier in &aggregator.multiplicative {
        final_value *= multiplier;
    }

    final_value
}

#[derive(Debug, Clone)]
pub struct Aggregator {
    additive: Vec<f64>,
    percent_additive: Vec<f64>,
    multiplicative: Vec<f64>,
    override_value: Option<f64>,
    executor: fn(&Aggregator, f64) -> f64,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self {
            additive: Vec::new(),
            percent_additive: Vec::new(),
            multiplicative: Vec::new(),
            override_value: None,
            executor: default_executor,
        }
    }
}

impl Aggregator {
    pub fn set_executor(&mut self, executor: Option<fn(&Aggregator, f64) -> f64>) {
        if let Some(executor) = executor {
            self.executor = executor;
        }
    }

    pub fn add_additive(&mut self, value: f64) {
        self.additive.push(value);
    }

    pub fn add_multiplicative(&mut self, value: f64) {
        self.multiplicative.push(value);
    }

    pub fn add_percent_additive(&mut self, value: f64) {
        self.percent_additive.push(value);
    }

    pub fn set_override(&mut self, value: f64) {
        self.override_value = Some(value);
    }

    pub fn reset(&mut self) {
        self.additive.clear();
        self.percent_additive.clear();
        self.multiplicative.clear();
        self.override_value = None;
    }

    pub fn evaluate(&mut self, base_value: f64) -> f64 {
        (self.executor)(self, base_value)
    }
}
