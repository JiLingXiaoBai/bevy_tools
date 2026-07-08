use super::common::{
    active_effect_handles, apply_effect, attribute_set, current_value, empty_effect_tags,
    register_attribute, run_effect_duration_tick, test_app,
};
use bevy::prelude::*;
use bevy_tools::{
    AttributeClamp, AttributeIdManager, AttributeSet, EffectDurationTicks, GameplayEffect,
    ModifierMagnitude, ModifierOperation, StackingPolicy, UniqueNamePool,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

static POST_EXECUTE_COUNT: AtomicUsize = AtomicUsize::new(0);

fn count_post_execute(
    _attributes: &mut AttributeSet,
    _id: bevy_tools::AttributeId,
    old_value: f64,
    new_value: f64,
) {
    assert_eq!(old_value, 10.0);
    assert_eq!(new_value, 15.0);
    POST_EXECUTE_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn duration_modifier_is_clamped_and_removed_on_expiration() {
    let mut app = test_app();
    let health = register_attribute(&mut app, "Health");
    let target = app
        .world_mut()
        .spawn(attribute_set(
            health,
            90.0,
            AttributeClamp::Range {
                min: Some(0.0),
                max: Some(100.0),
            },
        ))
        .id();
    let effect = Arc::new(GameplayEffect::new(
        vec![super::common::add_modifier(health, 20.0)],
        EffectDurationTicks::DurationTicks(ModifierMagnitude::Flat(2.0)),
        None,
        1.0,
        StackingPolicy::non_stacking(),
        empty_effect_tags(),
    ));

    assert!(apply_effect(&mut app, target, target, effect));
    assert_eq!(current_value(&mut app, target, health), 100.0);

    run_effect_duration_tick(&mut app);
    assert_eq!(current_value(&mut app, target, health), 100.0);

    run_effect_duration_tick(&mut app);
    assert_eq!(current_value(&mut app, target, health), 90.0);
    assert!(active_effect_handles(&app, target).is_empty());
}

#[test]
fn clamp_supports_min_only_max_only_and_closed_range() {
    let mut app = test_app();
    let min_only = register_attribute(&mut app, "MinOnly");
    let max_only = register_attribute(&mut app, "MaxOnly");
    let closed = register_attribute(&mut app, "Closed");

    let mut attributes = AttributeSet::default();
    attributes.initialize_attribute(
        min_only,
        -5.0,
        None,
        AttributeClamp::Range {
            min: Some(0.0),
            max: None,
        },
    );
    attributes.initialize_attribute(
        max_only,
        25.0,
        None,
        AttributeClamp::Range {
            min: None,
            max: Some(10.0),
        },
    );
    attributes.initialize_attribute(
        closed,
        20.0,
        None,
        AttributeClamp::Range {
            min: Some(0.0),
            max: Some(15.0),
        },
    );
    let entity = app.world_mut().spawn(attributes).id();

    assert_eq!(current_value(&mut app, entity, min_only), 0.0);
    assert_eq!(current_value(&mut app, entity, max_only), 10.0);
    assert_eq!(current_value(&mut app, entity, closed), 15.0);
}

#[test]
fn duration_modifiers_use_add_percent_then_multiply_order() {
    let mut app = test_app();
    let damage = register_attribute(&mut app, "Damage");
    let target = app
        .world_mut()
        .spawn(attribute_set(damage, 100.0, AttributeClamp::None))
        .id();
    let effect = Arc::new(GameplayEffect::new(
        vec![
            super::common::modifier(damage, ModifierOperation::Multiply, 2.0),
            super::common::modifier(damage, ModifierOperation::PercentAdd, 0.5),
            super::common::modifier(damage, ModifierOperation::Add, 10.0),
        ],
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        empty_effect_tags(),
    ));

    assert!(apply_effect(&mut app, target, target, effect));
    assert_eq!(current_value(&mut app, target, damage), 330.0);
}

#[test]
fn override_modifier_takes_precedence_over_other_duration_modifiers() {
    let mut app = test_app();
    let damage = register_attribute(&mut app, "Damage");
    let target = app
        .world_mut()
        .spawn(attribute_set(damage, 100.0, AttributeClamp::None))
        .id();
    let effect = Arc::new(GameplayEffect::new(
        vec![
            super::common::modifier(damage, ModifierOperation::Add, 10.0),
            super::common::modifier(damage, ModifierOperation::Multiply, 2.0),
            super::common::modifier(damage, ModifierOperation::Override, 42.0),
        ],
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        empty_effect_tags(),
    ));

    assert!(apply_effect(&mut app, target, target, effect));
    assert_eq!(current_value(&mut app, target, damage), 42.0);
}

#[test]
fn modifiers_targeting_uninitialized_attribute_do_not_create_values() {
    let mut app = test_app();
    let health = register_attribute(&mut app, "Health");
    let mana = register_attribute(&mut app, "Mana");
    let target = app
        .world_mut()
        .spawn(attribute_set(health, 10.0, AttributeClamp::None))
        .id();
    let effect = Arc::new(GameplayEffect::new(
        vec![super::common::add_modifier(mana, 5.0)],
        EffectDurationTicks::Instant,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        empty_effect_tags(),
    ));

    assert!(apply_effect(&mut app, target, target, effect));
    assert_eq!(current_value(&mut app, target, health), 10.0);
    assert!(
        app.world_mut()
            .entity_mut(target)
            .get_mut::<AttributeSet>()
            .unwrap()
            .get_current_value(mana)
            .is_none()
    );
}

#[test]
fn removing_missing_modifier_handle_does_not_change_current_value() {
    let mut app = test_app();
    let health = register_attribute(&mut app, "Health");
    let target = app
        .world_mut()
        .spawn(attribute_set(health, 10.0, AttributeClamp::None))
        .id();
    let effect = Arc::new(GameplayEffect::new(
        vec![super::common::add_modifier(health, 5.0)],
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        empty_effect_tags(),
    ));
    let missing_handle = app.world_mut().spawn_empty().id();

    assert!(apply_effect(&mut app, target, target, effect));
    app.world_mut()
        .entity_mut(target)
        .get_mut::<AttributeSet>()
        .unwrap()
        .remove_modifiers(missing_handle);

    assert_eq!(current_value(&mut app, target, health), 15.0);
}

#[test]
fn instant_modifier_invokes_post_execute_callback() {
    POST_EXECUTE_COUNT.store(0, Ordering::SeqCst);
    let mut app = test_app();
    let health = register_attribute(&mut app, "Health");
    let mut attributes = attribute_set(health, 10.0, AttributeClamp::None);
    attributes.set_post_execute(Some(count_post_execute));
    let target = app.world_mut().spawn(attributes).id();

    assert!(apply_effect(
        &mut app,
        target,
        target,
        super::common::instant_add_effect(health, 5.0),
    ));

    assert_eq!(POST_EXECUTE_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn attribute_id_registration_reports_capacity_exceeded() {
    let mut app = test_app();
    let result = {
        let unique_names: Vec<_> = {
            let mut names = app.world_mut().resource_mut::<UniqueNamePool>();
            (0..=bevy_tools::ATTRIBUTE_SET_SIZE)
                .map(|index| names.new_name(&format!("Attribute{index}")))
                .collect()
        };

        let mut manager = app.world_mut().resource_mut::<AttributeIdManager>();
        let mut result = Ok(());
        for unique_name in unique_names {
            if let Err(err) = manager.register_id_internal(unique_name) {
                result = Err(err);
                break;
            }
        }
        result
    };

    assert_eq!(
        result,
        Err(bevy_tools::AttributeIdError::CapacityExceeded {
            max: bevy_tools::ATTRIBUTE_SET_SIZE
        })
    );
}

#[test]
fn attribute_set_snapshot_captures_base_current_and_source_entity() {
    let mut app = test_app();
    let health = register_attribute(&mut app, "Health");
    let source = app
        .world_mut()
        .spawn(attribute_set(health, 10.0, AttributeClamp::None))
        .id();
    let effect = Arc::new(GameplayEffect::new(
        vec![super::common::add_modifier(health, 5.0)],
        EffectDurationTicks::Infinite,
        None,
        1.0,
        StackingPolicy::non_stacking(),
        empty_effect_tags(),
    ));
    assert!(apply_effect(&mut app, source, source, effect));

    let snapshot = app
        .world_mut()
        .entity_mut(source)
        .get_mut::<AttributeSet>()
        .unwrap()
        .make_snapshot(source);

    assert_eq!(snapshot.get_source_entity(), source);
    assert_eq!(snapshot.get_base_value(health), Some(10.0));
    assert_eq!(snapshot.get_current_value(health), Some(15.0));
}

#[test]
fn attribute_set_snapshot_is_not_changed_by_later_attribute_mutation() {
    let mut app = test_app();
    let health = register_attribute(&mut app, "Health");
    let source = app
        .world_mut()
        .spawn(attribute_set(health, 10.0, AttributeClamp::None))
        .id();
    let snapshot = app
        .world_mut()
        .entity_mut(source)
        .get_mut::<AttributeSet>()
        .unwrap()
        .make_snapshot(source);

    assert!(apply_effect(
        &mut app,
        source,
        source,
        super::common::instant_add_effect(health, 20.0),
    ));

    assert_eq!(current_value(&mut app, source, health), 30.0);
    assert_eq!(snapshot.get_base_value(health), Some(10.0));
    assert_eq!(snapshot.get_current_value(health), Some(10.0));
}
