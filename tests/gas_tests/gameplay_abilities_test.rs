use super::common::{
    ability_task_count, activate_ability, active_ability_count, add_tag_to_entity, attribute_set,
    current_value, effect_tags, give_ability, register_attribute, register_tag, run_ability_tasks,
    run_finished_ability_cleanup, spawn_ability_task, spawn_active_ability, test_app,
};
use bevy_tools::{
    AbilityActivationStatus, AbilitySpecHandle, AbilitySystemComponent, AbilityTags, AbilityTask,
    AbilityTaskDef, AbilityTaskOnFinished, AbilityTaskOnFinishedDef, EffectDurationTicks,
    GameplayAbility, GameplayTagContainer, StackingPolicy,
};
use std::sync::Arc;

#[test]
fn ability_activation_commits_cost_and_cooldown_then_cooldown_blocks_reactivation() {
    let mut app = test_app();
    let mana = register_attribute(&mut app, "Mana");
    let cooldown_tag = register_tag(&mut app, "Cooldown.Fireball");
    let source = app
        .world_mut()
        .spawn((
            AbilitySystemComponent::default(),
            GameplayTagContainer::default(),
            attribute_set(mana, 50.0, bevy_tools::AttributeClamp::None),
        ))
        .id();
    let cost = super::common::instant_add_effect(mana, -20.0);
    let cooldown = Arc::new(bevy_tools::GameplayEffect::new(
        Vec::new(),
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        effect_tags(Vec::new(), vec![cooldown_tag]),
    ));
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        Some(cooldown),
        Some(cost),
        Vec::new(),
        true,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(activate_ability(&mut app, source, source, handle));
    assert_eq!(current_value(&mut app, source, mana), 30.0);
    assert!(
        app.world()
            .entity(source)
            .get::<GameplayTagContainer>()
            .unwrap()
            .has_tag(&cooldown_tag)
    );

    run_finished_ability_cleanup(&mut app);
    assert_eq!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(handle)
            .unwrap()
            .get_active_count(),
        0
    );
    assert!(!activate_ability(&mut app, source, source, handle));
    assert_eq!(current_value(&mut app, source, mana), 30.0);
}

#[test]
fn ability_cost_fails_when_it_would_drop_attribute_below_zero() {
    let mut app = test_app();
    let stamina = register_attribute(&mut app, "Stamina");
    let source = app
        .world_mut()
        .spawn((
            AbilitySystemComponent::default(),
            attribute_set(stamina, 10.0, bevy_tools::AttributeClamp::None),
        ))
        .id();
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        Some(super::common::instant_add_effect(stamina, -20.0)),
        Vec::new(),
        true,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(!activate_ability(&mut app, source, source, handle));
    assert_eq!(current_value(&mut app, source, stamina), 10.0);
}

#[test]
fn wait_ticks_task_marks_active_ability_ending_after_delay() {
    let mut app = test_app();
    let source = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();
    let handle = AbilitySpecHandle::new(7);
    let active_ability = spawn_active_ability(&mut app, source, target, handle);
    spawn_ability_task(
        &mut app,
        AbilityTask::wait_ticks(active_ability, 2, AbilityTaskOnFinished::EndAbility),
    );

    run_ability_tasks(&mut app);
    assert_eq!(
        app.world()
            .entity(active_ability)
            .get::<bevy_tools::ActiveGameplayAbility>()
            .unwrap()
            .get_status(),
        AbilityActivationStatus::Active
    );

    run_ability_tasks(&mut app);
    assert_eq!(
        app.world()
            .entity(active_ability)
            .get::<bevy_tools::ActiveGameplayAbility>()
            .unwrap()
            .get_status(),
        AbilityActivationStatus::Ending
    );
}

#[test]
fn ability_disallows_multiple_instances_by_default() {
    let mut app = test_app();
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(activate_ability(&mut app, source, source, handle));
    assert!(!activate_ability(&mut app, source, source, handle));
    assert_eq!(active_ability_count(&mut app), 1);
    assert_eq!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(handle)
            .unwrap()
            .get_active_count(),
        1
    );
}

#[test]
fn ability_allows_multiple_instances_when_enabled() {
    let mut app = test_app();
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        true,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(activate_ability(&mut app, source, source, handle));
    assert!(activate_ability(&mut app, source, source, handle));
    assert_eq!(active_ability_count(&mut app), 2);
    assert_eq!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(handle)
            .unwrap()
            .get_active_count(),
        2
    );
}

#[test]
fn ability_activation_required_and_blocked_tags_are_enforced() {
    let mut app = test_app();
    let required = register_tag(&mut app, "State.Weapon.Ready");
    let blocked = register_tag(&mut app, "State.Silenced");
    let source = app
        .world_mut()
        .spawn((
            AbilitySystemComponent::default(),
            GameplayTagContainer::default(),
        ))
        .id();
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![required],
            vec![blocked],
        ),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        true,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(!activate_ability(&mut app, source, source, handle));

    add_tag_to_entity(&mut app, source, required);
    assert!(activate_ability(&mut app, source, source, handle));

    add_tag_to_entity(&mut app, source, blocked);
    assert!(!activate_ability(&mut app, source, source, handle));
}

#[test]
fn active_ability_block_tags_prevent_matching_ability_activation() {
    let mut app = test_app();
    let channel_tag = register_tag(&mut app, "Ability.Channel");
    let movement_tag = register_tag(&mut app, "Ability.Movement");
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let channel = Arc::new(GameplayAbility::new(
        AbilityTags::new(
            vec![channel_tag],
            Vec::new(),
            vec![movement_tag],
            Vec::new(),
            Vec::new(),
        ),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        false,
    ));
    let movement = Arc::new(GameplayAbility::new(
        AbilityTags::new(
            vec![movement_tag],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        false,
    ));
    let channel_handle = give_ability(&mut app, source, channel);
    let movement_handle = give_ability(&mut app, source, movement);

    assert!(activate_ability(&mut app, source, source, channel_handle));
    assert!(!activate_ability(&mut app, source, source, movement_handle));
}

#[test]
fn activating_ability_cancels_matching_active_abilities() {
    let mut app = test_app();
    let stance_tag = register_tag(&mut app, "Ability.Stance");
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let stance = Arc::new(GameplayAbility::new(
        AbilityTags::new(
            vec![stance_tag],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        false,
    ));
    let breaker = Arc::new(GameplayAbility::new(
        AbilityTags::new(
            Vec::new(),
            vec![stance_tag],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        false,
    ));
    let stance_handle = give_ability(&mut app, source, stance);
    let breaker_handle = give_ability(&mut app, source, breaker);

    assert!(activate_ability(&mut app, source, source, stance_handle));
    assert_eq!(active_ability_count(&mut app), 1);
    assert!(activate_ability(&mut app, source, source, breaker_handle));
    assert_eq!(active_ability_count(&mut app), 1);
    assert_eq!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(stance_handle)
            .unwrap()
            .get_active_count(),
        0
    );
}

#[test]
fn cooldown_prepare_failure_does_not_spend_ability_cost() {
    let mut app = test_app();
    let mana = register_attribute(&mut app, "Mana");
    let cooldown_tag = register_tag(&mut app, "Cooldown.NoContainer");
    let source = app
        .world_mut()
        .spawn((
            AbilitySystemComponent::default(),
            attribute_set(mana, 50.0, bevy_tools::AttributeClamp::None),
        ))
        .id();
    let cooldown = Arc::new(bevy_tools::GameplayEffect::new(
        Vec::new(),
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        effect_tags(Vec::new(), vec![cooldown_tag]),
    ));
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        Some(cooldown),
        Some(super::common::instant_add_effect(mana, -20.0)),
        Vec::new(),
        true,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(!activate_ability(&mut app, source, source, handle));
    assert_eq!(current_value(&mut app, source, mana), 50.0);
    assert_eq!(active_ability_count(&mut app), 0);
}

#[test]
fn activation_effect_failure_does_not_block_ability_success() {
    let mut app = test_app();
    let tag = register_tag(&mut app, "Effect.MissingTargetContainer");
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let best_effort_effect = Arc::new(bevy_tools::GameplayEffect::new(
        Vec::new(),
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        effect_tags(Vec::new(), vec![tag]),
    ));
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        None,
        vec![best_effort_effect],
        true,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(activate_ability(&mut app, source, source, handle));
    assert_eq!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(handle)
            .unwrap()
            .get_active_count(),
        1
    );
}

#[test]
fn cleanup_finished_ability_despawns_startup_tasks_and_is_repeatable() {
    let mut app = test_app();
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        vec![AbilityTaskDef::wait_ticks(
            10,
            AbilityTaskOnFinishedDef::None,
        )],
        None,
        None,
        Vec::new(),
        true,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(activate_ability(&mut app, source, source, handle));
    assert_eq!(active_ability_count(&mut app), 1);
    assert_eq!(ability_task_count(&mut app), 1);

    run_finished_ability_cleanup(&mut app);
    assert_eq!(active_ability_count(&mut app), 0);
    assert_eq!(ability_task_count(&mut app), 0);
    assert_eq!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(handle)
            .unwrap()
            .get_active_count(),
        0
    );

    run_finished_ability_cleanup(&mut app);
    assert_eq!(active_ability_count(&mut app), 0);
    assert_eq!(ability_task_count(&mut app), 0);
}

#[test]
fn ability_spec_preserves_input_id_and_clear_rebuilds_indices() {
    let mut asc = AbilitySystemComponent::default();
    let first = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
        false,
    ));
    let second = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
        false,
    ));

    let first_handle = asc.give_ability(first, 2, Some(4));
    let second_handle = asc.give_ability(second, 3, Some(8));

    assert_eq!(
        asc.find_ability_spec(first_handle).unwrap().get_input_id(),
        Some(4)
    );
    assert_eq!(
        asc.find_ability_spec(second_handle).unwrap().get_input_id(),
        Some(8)
    );
    assert!(asc.clear_ability(first_handle));
    assert!(asc.find_ability_spec(first_handle).is_none());
    assert_eq!(asc.find_ability_spec(second_handle).unwrap().get_level(), 3);
    assert_eq!(
        asc.find_ability_spec(second_handle).unwrap().get_input_id(),
        Some(8)
    );
}

#[test]
fn clear_ability_returns_false_while_spec_is_active() {
    let mut app = test_app();
    let source = app
        .world_mut()
        .spawn(AbilitySystemComponent::default())
        .id();
    let ability = Arc::new(GameplayAbility::new(
        AbilityTags::default(),
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
        false,
    ));
    let handle = give_ability(&mut app, source, ability);

    assert!(activate_ability(&mut app, source, source, handle));
    assert!(
        !app.world_mut()
            .entity_mut(source)
            .get_mut::<AbilitySystemComponent>()
            .unwrap()
            .clear_ability(handle)
    );
    assert!(
        app.world()
            .entity(source)
            .get::<AbilitySystemComponent>()
            .unwrap()
            .find_ability_spec(handle)
            .is_some()
    );
}
