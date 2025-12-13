#[derive(Debug, Clone, Copy)]
pub struct Attribute {
    base: f64,
    current: f64,
    clamp_min: Option<f64>,
    clamp_max: Option<f64>,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base: 0.0,
            current: 0.0,
            clamp_min: None,
            clamp_max: None,
        }
    }
}

impl Attribute {
    fn clamp(&mut self) {
        if let Some(min) = self.clamp_min {
            self.current = self.current.max(min);
        }
        if let Some(max) = self.clamp_max {
            self.current = self.current.min(max);
        }
    }

    pub fn init(&mut self, base_value: f64, clamp_min: Option<f64>, clamp_max: Option<f64>) {
        self.base = base_value;
        self.current = base_value;
        self.clamp_min = clamp_min;
        self.clamp_max = clamp_max;
        self.clamp();
    }
}
