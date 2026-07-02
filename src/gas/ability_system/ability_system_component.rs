use crate::apply_gameplay_effect;
use crate::attributes::{AttributeSet, AttributeSetSnapshot};
use crate::gameplay_abilities::{
    AbilityActivationStatus, AbilitySpecHandle, ActiveAbilityHandle, ActiveGameplayAbility,
    GameplayAbility, GameplayAbilitySpec,
};
use crate::gameplay_effects::{ActiveGameplayEffect, EffectContext};
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
    pub active_effect_query: Query<'w, 's, (Entity, &'static ActiveGameplayEffect)>,
    pub active_ability_query: Query<'w, 's, (Entity, &'static ActiveGameplayAbility)>,
    pub time: Res<'w, Time>,
}

#[derive(Component, Default)]
pub struct AbilitySystemComponent {
    next_ability_handle: u32,
    abilities: Vec<GameplayAbilitySpec>,
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

    pub fn clear_ability(&mut self, handle: AbilitySpecHandle) -> bool {
        if self
            .find_ability_spec(handle)
            .is_some_and(|spec| spec.get_active_count() > 0)
        {
            return false;
        }

        let old_len = self.abilities.len();
        self.abilities.retain(|spec| spec.get_handle() != handle);
        old_len != self.abilities.len()
    }

    pub fn get_ability_specs(&self) -> &[GameplayAbilitySpec] {
        &self.abilities
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
        source: Entity,
        target: Entity,
        spec_handle: AbilitySpecHandle,
        ability: &GameplayAbility,
        commands: &mut Commands,
        tag_manager: &Res<GameplayTagManager>,
    ) -> ActiveAbilityHandle {
        self.blocked_ability_tags.add_tags(
            ability.get_tags().get_block_abilities_with_tags(),
            tag_manager,
        );

        if let Some(spec) = self.find_ability_spec_mut(spec_handle) {
            spec.increment_active_count();
        }

        let mut entity_cmds = commands.spawn(ActiveGameplayAbility::new(
            source,
            spec_handle,
            target,
            AbilityActivationStatus::Active,
        ));
        let active_handle = entity_cmds.id();
        entity_cmds.set_parent_in_place(source);

        active_handle
    }

    fn finish_active_ability(
        &mut self,
        active_handle: ActiveAbilityHandle,
        active_ability: &ActiveGameplayAbility,
        commands: &mut Commands,
        tag_manager: &Res<GameplayTagManager>,
    ) -> bool {
        self.rollback_started_ability(
            active_handle,
            active_ability.get_spec_handle(),
            commands,
            tag_manager,
        )
    }

    fn rollback_started_ability(
        &mut self,
        active_handle: ActiveAbilityHandle,
        spec_handle: AbilitySpecHandle,
        commands: &mut Commands,
        tag_manager: &Res<GameplayTagManager>,
    ) -> bool {
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
        commands.entity(active_handle).despawn();
        true
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

    cancel_active_abilities_with_tags(
        source,
        ability.get_tags().get_cancel_abilities_with_tags(),
        params,
    );

    let active_handle = {
        let Ok(mut asc) = params.asc_query.get_mut(source) else {
            return false;
        };
        asc.start_ability(
            source,
            target,
            handle,
            &ability,
            &mut params.commands,
            &params.tag_manager,
        )
    };

    if !commit_ability(source, target, &ability, level, params) {
        if let Ok(mut asc) = params.asc_query.get_mut(source) {
            asc.rollback_started_ability(
                active_handle,
                handle,
                &mut params.commands,
                &params.tag_manager,
            );
        }
        return false;
    }

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
    finish_ability_with_status(
        source,
        active_handle,
        AbilityActivationStatus::Ending,
        params,
    )
}

pub fn cancel_ability(
    source: Entity,
    active_handle: ActiveAbilityHandle,
    params: &mut AbilitySystemParams,
) -> bool {
    finish_ability_with_status(
        source,
        active_handle,
        AbilityActivationStatus::Cancelled,
        params,
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
) -> bool {
    if let Some(cost_def) = ability.get_cost()
        && !apply_gameplay_effect(source, source, cost_def, params, level)
    {
        return false;
    }

    if let Some(cooldown_def) = ability.get_cooldown()
        && !apply_gameplay_effect(source, source, cooldown_def, params, level)
    {
        return false;
    }

    true
}

fn finish_ability_with_status(
    source: Entity,
    active_handle: ActiveAbilityHandle,
    _status: AbilityActivationStatus,
    params: &mut AbilitySystemParams,
) -> bool {
    let Ok((_, active_ability)) = params.active_ability_query.get(active_handle) else {
        return false;
    };
    if active_ability.get_source() != source {
        return false;
    }
    let active_ability = active_ability.clone();

    let Ok(mut asc) = params.asc_query.get_mut(source) else {
        return false;
    };
    asc.finish_active_ability(
        active_handle,
        &active_ability,
        &mut params.commands,
        &params.tag_manager,
    )
}

fn cancel_active_abilities_with_tags(
    source: Entity,
    tags: &[GameplayTag],
    params: &mut AbilitySystemParams,
) {
    if tags.is_empty() {
        return;
    }

    let active_handles: Vec<_> = {
        let Ok(asc) = params.asc_query.get(source) else {
            return;
        };
        params
            .active_ability_query
            .iter()
            .filter_map(|(active_handle, active)| {
                if active.get_source() != source {
                    return None;
                }
                let spec = asc.find_ability_spec(active.get_spec_handle())?;
                ability_has_any_tags(spec.get_ability(), tags, &params.tag_manager)
                    .then_some(active_handle)
            })
            .collect()
    };

    for active_handle in active_handles {
        cancel_ability(source, active_handle, params);
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

    let Ok(attr_set) = params.attr_set_query.get(source) else {
        return false;
    };

    let cost_spec = cost_def.make_spec(&context);
    for cost in cost_spec.get_modifier_specs() {
        let Some(current_val) = attr_set.get_current_value(cost.get_id()) else {
            return false;
        };
        if current_val + cost.get_value() < 0.0 {
            return false;
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
