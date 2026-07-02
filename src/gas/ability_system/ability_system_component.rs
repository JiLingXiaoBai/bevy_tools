use crate::apply_gameplay_effect;
use crate::attributes::{AttributeSet, AttributeSetSnapshot};
use crate::gameplay_abilities::{
    AbilityActivationStatus, AbilitySpecHandle, ActiveAbilityHandle, ActiveGameplayAbility,
    GameplayAbility, GameplayAbilitySpec,
};
use crate::gameplay_effects::EffectContext;
use crate::gameplay_tags::{GameplayTag, GameplayTagContainer, GameplayTagManager};
use crate::randoms::Random;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(SystemParam)]
pub struct AbilitySystemParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub tag_manager: Res<'w, GameplayTagManager>,
    pub random_gen: ResMut<'w, Random>,
    pub attr_set_query: Query<'w, 's, &'static mut AttributeSet>,
    pub tag_container_query: Query<'w, 's, &'static mut GameplayTagContainer>,
    pub asc_query: Query<'w, 's, &'static mut AbilitySystemComponent>,
    pub attr_set_snapshot_query: Query<'w, 's, &'static AttributeSetSnapshot>,
    pub time: Res<'w, Time>,
}

#[derive(Component, Default)]
pub struct AbilitySystemComponent {
    next_ability_handle: u32,
    next_active_ability_handle: u32,
    abilities: Vec<GameplayAbilitySpec>,
    active_abilities: Vec<ActiveGameplayAbility>,
    blocked_ability_tags: GameplayTagContainer,
}

impl AbilitySystemComponent {
    pub fn give_ability(
        &mut self,
        ability: Arc<GameplayAbility>,
        level: u32,
        input_id: Option<u16>,
    ) -> AbilitySpecHandle {
        let handle = AbilitySpecHandle::new(self.next_ability_handle);
        self.next_ability_handle = self.next_ability_handle.wrapping_add(1);
        self.abilities
            .push(GameplayAbilitySpec::new(handle, ability, level, input_id));
        handle
    }

    pub fn clear_ability(
        &mut self,
        handle: AbilitySpecHandle,
        tag_manager: &Res<GameplayTagManager>,
    ) -> bool {
        let active_handles: Vec<_> = self
            .active_abilities
            .iter()
            .filter(|active| active.get_spec_handle() == handle)
            .map(|active| active.get_handle())
            .collect();
        for active_handle in active_handles {
            self.finish_active_ability(
                active_handle,
                AbilityActivationStatus::Cancelled,
                tag_manager,
            );
        }

        let old_len = self.abilities.len();
        self.abilities.retain(|spec| spec.get_handle() != handle);
        old_len != self.abilities.len()
    }

    pub fn get_ability_specs(&self) -> &[GameplayAbilitySpec] {
        &self.abilities
    }

    pub fn get_active_abilities(&self) -> &[ActiveGameplayAbility] {
        &self.active_abilities
    }

    pub fn get_blocked_ability_tags(&self) -> &GameplayTagContainer {
        &self.blocked_ability_tags
    }

    pub fn find_ability_spec(&self, handle: AbilitySpecHandle) -> Option<&GameplayAbilitySpec> {
        self.abilities
            .iter()
            .find(|spec| spec.get_handle() == handle)
    }

    fn find_ability_spec_mut(
        &mut self,
        handle: AbilitySpecHandle,
    ) -> Option<&mut GameplayAbilitySpec> {
        self.abilities
            .iter_mut()
            .find(|spec| spec.get_handle() == handle)
    }

    fn start_ability(
        &mut self,
        target: Entity,
        spec_handle: AbilitySpecHandle,
        ability: &GameplayAbility,
        tag_manager: &Res<GameplayTagManager>,
    ) -> ActiveAbilityHandle {
        self.cancel_active_abilities_with_tags(
            ability.get_tags().get_cancel_abilities_with_tags(),
            tag_manager,
        );
        self.blocked_ability_tags.add_tags(
            ability.get_tags().get_block_abilities_with_tags(),
            tag_manager,
        );

        let active_handle = ActiveAbilityHandle::new(self.next_active_ability_handle);
        self.next_active_ability_handle = self.next_active_ability_handle.wrapping_add(1);

        if let Some(spec) = self.find_ability_spec_mut(spec_handle) {
            spec.increment_active_count();
        }

        self.active_abilities.push(ActiveGameplayAbility::new(
            active_handle,
            spec_handle,
            target,
            AbilityActivationStatus::Active,
        ));

        active_handle
    }

    fn finish_active_ability(
        &mut self,
        active_handle: ActiveAbilityHandle,
        status: AbilityActivationStatus,
        tag_manager: &Res<GameplayTagManager>,
    ) -> bool {
        let Some(index) = self
            .active_abilities
            .iter()
            .position(|active| active.get_handle() == active_handle)
        else {
            return false;
        };

        self.active_abilities[index].set_status(status);
        let spec_handle = self.active_abilities[index].get_spec_handle();
        let blocked_tags = self
            .find_ability_spec(spec_handle)
            .map(|spec| {
                spec.get_ability()
                    .get_tags()
                    .get_block_abilities_with_tags()
                    .to_vec()
            })
            .unwrap_or_default();

        if let Some(spec) = self.find_ability_spec_mut(spec_handle) {
            spec.decrement_active_count();
        }

        self.blocked_ability_tags
            .remove_tags(&blocked_tags, tag_manager);
        self.active_abilities.swap_remove(index);
        true
    }

    fn cancel_active_abilities_with_tags(
        &mut self,
        tags: &[GameplayTag],
        tag_manager: &Res<GameplayTagManager>,
    ) {
        if tags.is_empty() {
            return;
        }

        let active_handles: Vec<_> = self
            .active_abilities
            .iter()
            .filter_map(|active| {
                let spec = self.find_ability_spec(active.get_spec_handle())?;
                if ability_has_any_tags(spec.get_ability(), tags, tag_manager) {
                    Some(active.get_handle())
                } else {
                    None
                }
            })
            .collect();

        for active_handle in active_handles {
            self.finish_active_ability(
                active_handle,
                AbilityActivationStatus::Cancelled,
                tag_manager,
            );
        }
    }
}

pub fn try_activate_ability_by_handle(
    source: Entity,
    target: Entity,
    handle: AbilitySpecHandle,
    params: &mut AbilitySystemParams,
) -> bool {
    let (ability, level, active_count) = {
        let Ok(asc) = params.asc_query.get(source) else {
            return false;
        };
        let Some(spec) = asc.find_ability_spec(handle) else {
            return false;
        };
        (
            spec.get_ability().clone(),
            spec.get_level(),
            spec.get_active_count(),
        )
    };

    if !ability.allow_multiple_instances() && active_count > 0 {
        return false;
    }

    if !can_activate_ability(source, target, &ability, level, params) {
        return false;
    }

    let active_handle = {
        let Ok(mut asc) = params.asc_query.get_mut(source) else {
            return false;
        };
        asc.start_ability(target, handle, &ability, &params.tag_manager)
    };

    commit_ability(source, target, &ability, level, params);

    for effect in ability.get_activation_effects() {
        apply_gameplay_effect(source, target, effect, params, level);
    }

    if ability.should_end_on_activation() {
        end_ability(source, active_handle, params);
    }

    true
}

pub fn end_ability(
    source: Entity,
    active_handle: ActiveAbilityHandle,
    params: &mut AbilitySystemParams,
) -> bool {
    let Ok(mut asc) = params.asc_query.get_mut(source) else {
        return false;
    };
    asc.finish_active_ability(
        active_handle,
        AbilityActivationStatus::Ending,
        &params.tag_manager,
    )
}

pub fn cancel_ability(
    source: Entity,
    active_handle: ActiveAbilityHandle,
    params: &mut AbilitySystemParams,
) -> bool {
    let Ok(mut asc) = params.asc_query.get_mut(source) else {
        return false;
    };
    asc.finish_active_ability(
        active_handle,
        AbilityActivationStatus::Cancelled,
        &params.tag_manager,
    )
}

pub fn can_activate_ability(
    source: Entity,
    target: Entity,
    ability: &Arc<GameplayAbility>,
    level: u32,
    params: &AbilitySystemParams,
) -> bool {
    if let Ok(asc) = params.asc_query.get(source)
        && asc
            .blocked_ability_tags
            .has_any(ability.get_tags().get_ability_asset_tags())
    {
        return false;
    }

    let source_tags = params.tag_container_query.get(source).ok();
    if let Some(tags) = source_tags {
        let ability_tags = ability.get_tags();
        if tags.has_any(ability_tags.get_activation_blocked_tags()) {
            return false;
        }
        if !tags.has_all(ability_tags.get_activation_required_tags()) {
            return false;
        }

        if let Some(cooldown_def) = ability.get_cooldown()
            && tags.has_any(cooldown_def.get_tags().get_granted_tags())
        {
            return false;
        }
    }

    can_pay_ability_cost(source, target, ability, level, params)
}

pub fn commit_ability(
    source: Entity,
    _target: Entity,
    ability: &Arc<GameplayAbility>,
    level: u32,
    params: &mut AbilitySystemParams,
) {
    if let Some(cost_def) = ability.get_cost() {
        apply_gameplay_effect(source, source, cost_def, params, level);
    }

    if let Some(cooldown_def) = ability.get_cooldown() {
        apply_gameplay_effect(source, source, cooldown_def, params, level);
    }
}

fn can_pay_ability_cost(
    source: Entity,
    target: Entity,
    ability: &Arc<GameplayAbility>,
    level: u32,
    params: &AbilitySystemParams,
) -> bool {
    let Some(cost_def) = ability.get_cost() else {
        return true;
    };

    let context = EffectContext {
        source: Some(source),
        target: Some(target),
        attr_set_query: &params.attr_set_query.as_readonly(),
        tag_container_query: &params.tag_container_query.as_readonly(),
        asc_query: &params.asc_query.as_readonly(),
        attr_set_snapshot: params.attr_set_snapshot_query.get(source).ok(),
        level,
    };

    let cost_spec = cost_def.make_spec(&context);
    if let Ok(attr_set) = params.attr_set_query.get(source) {
        for cost in cost_spec.get_modifier_specs() {
            let current_val = attr_set.get_current_value(cost.get_id()).unwrap_or(0.0);
            if current_val + cost.get_value() < 0.0 {
                return false;
            }
        }
    }

    true
}

fn ability_has_any_tags(
    ability: &GameplayAbility,
    tags: &[GameplayTag],
    tag_manager: &Res<GameplayTagManager>,
) -> bool {
    let mut ability_tags = GameplayTagContainer::default();
    ability_tags.add_tags(ability.get_tags().get_ability_asset_tags(), tag_manager);
    ability_tags.has_any(tags)
}
