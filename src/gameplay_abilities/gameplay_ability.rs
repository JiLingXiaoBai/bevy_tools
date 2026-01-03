use crate::gameplay_effects::GameplayEffect;
use crate::gameplay_tags::GameplayTag;
use std::sync::Arc;

pub struct AbilityTags {
    ability_asset_tags: Vec<GameplayTag>,
    cancel_abilities_with_tags: Vec<GameplayTag>,
    block_abilities_with_tags: Vec<GameplayTag>,
    activation_required_tags: Vec<GameplayTag>,
    activation_blocked_tags: Vec<GameplayTag>,
}

impl AbilityTags {
    pub fn get_ability_asset_tags(&self) -> &Vec<GameplayTag> {
        &self.ability_asset_tags
    }

    pub fn get_cancel_abilities_with_tags(&self) -> &Vec<GameplayTag> {
        &self.cancel_abilities_with_tags
    }

    pub fn get_block_abilities_with_tags(&self) -> &Vec<GameplayTag> {
        &self.block_abilities_with_tags
    }

    pub fn get_activation_required_tags(&self) -> &Vec<GameplayTag> {
        &self.activation_required_tags
    }

    pub fn get_activation_blocked_tags(&self) -> &Vec<GameplayTag> {
        &self.activation_blocked_tags
    }
}

pub struct GameplayAbility {
    ability_tags: AbilityTags,
    _cooldown: Option<Arc<GameplayEffect>>,
    _cost: Option<Arc<GameplayEffect>>,
}

impl GameplayAbility {
    pub fn get_tags(&self) -> &AbilityTags {
        &self.ability_tags
    }
}
