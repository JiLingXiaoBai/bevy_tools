use super::super::attribute::{AttributeId, AttributeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierOperation {
    Add,
    Multiply,
    Divide,
    Override,
}

pub enum ModifierMagnitude{
    Flat(f64),
    Calculated(Box<dyn ModifierMagnitudeCalculation>)
}

pub trait ModifierMagnitudeCalculation: Send + Sync{
    // fn calculate(&self, source: &AttributeSet, target: &AttributeSet) -> f64;
}

pub struct Modifier {
    pub id: AttributeId,
    pub op: ModifierOperation,
    pub magnitude: ModifierMagnitude,
}
