pub struct GameplayAbilitySystemSettings;

impl GameplayAbilitySystemSettings {
    pub const ATTRIBUTE_SET_SIZE: usize = 256;
    pub const GAMEPLAY_TAG_SIZE: usize = 512;
    pub const ABILITY_ACTIVATION_QUEUE_MAX_PER_TICK: usize = 64;
    pub const GAMEPLAY_EFFECT_APPLICATION_QUEUE_MAX_PER_TICK: usize = 64;
    pub const ABILITY_CHAIN_MAX_DEPTH: u8 = 8;
}
