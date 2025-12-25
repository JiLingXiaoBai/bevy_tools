pub fn default_executor(aggregator: &Aggregator, base_value: f64) -> f64 {
    if !aggregator.dirty {
        return aggregator.cached;
    }
    if let Some(override_value) = aggregator.override_value {
        return override_value;
    }
    let mut value = base_value;
    for &add in &aggregator.additive {
        value += add;
    }
    for &mul in &aggregator.multiplicative {
        value *= mul;
    }
    value
}

#[derive(Clone)]
pub struct Aggregator {
    additive: Vec<f64>,
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
            multiplicative: Vec::new(),
            override_value: None,
            cached: 0.0,
            dirty: true,
            executor,
        }
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

    pub fn set_override(&mut self, value: f64) {
        self.override_value = Some(value);
        self.make_dirty();
    }

    pub fn clear(&mut self) {
        self.additive.clear();
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
