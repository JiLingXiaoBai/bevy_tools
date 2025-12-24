use crate::{GameplayTag, modifiers::Modifier};

pub enum EffectDuration {
    Instant,
    Duration(f32),
    Infinite,
}

pub struct EffectTags {
    pub granted_tags: Vec<GameplayTag>,
    pub required_tags: Vec<GameplayTag>,
    pub blocked_tags: Vec<GameplayTag>,
}

pub struct GameplayEffect {
    pub modifiers: Vec<Modifier>,
    pub duration: EffectDuration,
    pub tags: EffectTags,
}
