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
    cached: f64,
    dirty: bool,
    executor: fn(&Aggregator, f64) -> f64,
}

impl Aggregator {
    pub fn new(executor: fn(&Aggregator, f64) -> f64) -> Self {
        Self {
            additive: Vec::new(),
            percent_additive: Vec::new(),
            multiplicative: Vec::new(),
            override_value: None,
            cached: 0.0,
            dirty: true,
            executor,
        }
    }

    pub fn set_executor(&mut self, executor: fn(&Aggregator, f64) -> f64) {
        self.executor = executor;
        self.make_dirty();
    }

    #[inline]
    pub fn make_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn add_additive(&mut self, value: f64) {
        self.additive.push(value);
        self.make_dirty();
    }

    pub fn add_multiplicative(&mut self, value: f64) {
        self.multiplicative.push(value);
        self.make_dirty();
    }

    pub fn add_percent_additive(&mut self, value: f64) {
        self.percent_additive.push(value);
        self.make_dirty();
    }

    pub fn set_override(&mut self, value: f64) {
        self.override_value = Some(value);
        self.make_dirty();
    }

    pub fn reset(&mut self) {
        self.additive.clear();
        self.percent_additive.clear();
        self.multiplicative.clear();
        self.override_value = None;
        self.make_dirty();
    }

    pub fn evaluate(&mut self, base_value: f64) -> f64 {
        if self.dirty {
            self.cached = (self.executor)(self, base_value);
            self.dirty = false;
        }
        self.cached
    }
}
