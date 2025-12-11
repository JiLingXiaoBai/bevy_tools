#[derive(Debug, Clone, Copy)]
pub struct Attribute {
    pub base: f64,
    pub current: f64,
    pub clamp_min: Option<f64>,
    pub clamp_max: Option<f64>,
}

impl Default for Attribute {
    fn default() -> Self {
        Self{
            base: 0.0,
            current: 0.0,
            clamp_min: None,
            clamp_max: None,
        }
    }
}

impl Attribute {
    pub fn clamp(&mut self){
        if let Some(min) = self.clamp_min {
            self.current = self.current.max(min);
        }
        if let Some(max) = self.clamp_max {
            self.current = self.current.min(max);
        }
    }
}
