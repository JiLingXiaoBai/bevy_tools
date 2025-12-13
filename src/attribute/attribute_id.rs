#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum AttributeId {
    Null = 0,
    Health = 1,
}

impl AttributeId {
    pub fn to_index(self) -> usize {
        self as usize
    }
}
