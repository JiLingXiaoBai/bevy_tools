#[derive(Debug, Clone, Copy)]
pub struct AttributeSnapshot {
    base: f64,
    current: f64,
}

impl AttributeSnapshot {
    pub fn new(base: f64, current: f64) -> Self {
        Self { base, current }
    }

    pub fn base(&self) -> f64 {
        self.base
    }

    pub fn current(&self) -> f64 {
        self.current
    }
}
