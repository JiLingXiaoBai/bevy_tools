#[derive(Debug, Clone)]
pub struct Aggregator {
    additive: Vec<f64>,
    multiplicative: Vec<f64>,
    override_value: Option<f64>,
    cached: f64,
    dirty: bool,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self {
            additive: Vec::new(),
            multiplicative: Vec::new(),
            override_value: None,
            cached: 0.0,
            dirty: true,
        }
    }
}

impl Aggregator {
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
        if !self.dirty {
            return self.cached;
        }

        let mut value = base_value;

        for add in &self.additive {
            value += add;
        }

        for mul in &self.multiplicative {
            value *= mul;
        }

        if let Some(override_value) = self.override_value {
            value = override_value;
        }
        self.cached = value;
        self.dirty = false;
        value
    }
}
