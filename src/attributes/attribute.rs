#[derive(Debug, Clone, Copy)]
pub struct Attribute {
    base: f64,
    current: f64,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base: 0.0,
            current: 0.0,
        }
    }
}

impl Attribute {
    pub fn init(&mut self, base_value: f64) {
        self.base = base_value;
        self.current = base_value;
    }
}
