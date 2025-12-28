use std::sync::Arc;

use crate::gameplay_effects::GameplayEffect;
use crate::gameplay_tags::GameplayTag;

pub struct AbilityTags {
    _ability_asset_tags: Vec<GameplayTag>,
    _cancel_abilities_with_tags: Vec<GameplayTag>,
    _block_abilities_with_tags: Vec<GameplayTag>,
}

pub struct GameplayAbility {
    _ability_tags: AbilityTags,
    _cooldown: Option<Arc<GameplayEffect>>,
    _cost: Option<Arc<GameplayEffect>>,
}

#[derive(Clone)]
pub struct GameplayAbilitySpec {
    _def: Arc<GameplayAbility>,
}
