use crate::ability_system::AbilityActivationQueue;
use crate::gameplay_abilities::{AbilityActivationStatus, AbilitySpecHandle, ActiveAbilityHandle};
use crate::gameplay_effects::{EffectPayload, GameplayEffect, GameplayEffectApplicationQueue};
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub enum AbilityTaskOnFinished {
    None,
    EndAbility,
    ActivateAbility {
        source: Entity,
        target: Entity,
        handle: AbilitySpecHandle,
    },
    ApplyGameplayEffect {
        source: Entity,
        target: Entity,
        effect: Arc<GameplayEffect>,
        level: u32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum AbilityTaskKind {
    WaitTicks { remaining_ticks: u32 },
}

#[derive(Component, Clone)]
pub struct AbilityTask {
    active_ability: ActiveAbilityHandle,
    kind: AbilityTaskKind,
    on_finished: AbilityTaskOnFinished,
}

impl AbilityTask {
    pub fn wait_ticks(
        active_ability: ActiveAbilityHandle,
        ticks: u32,
        on_finished: AbilityTaskOnFinished,
    ) -> Self {
        Self {
            active_ability,
            kind: AbilityTaskKind::WaitTicks {
                remaining_ticks: ticks,
            },
            on_finished,
        }
    }

    pub fn get_active_ability(&self) -> ActiveAbilityHandle {
        self.active_ability
    }

    pub fn get_kind(&self) -> AbilityTaskKind {
        self.kind
    }

    pub fn get_on_finished(&self) -> &AbilityTaskOnFinished {
        &self.on_finished
    }

    fn tick(&mut self) -> bool {
        match &mut self.kind {
            AbilityTaskKind::WaitTicks { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    *remaining_ticks -= 1;
                }
                *remaining_ticks == 0
            }
        }
    }
}

pub fn tick_ability_tasks_system(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut AbilityTask)>,
    mut active_ability_query: Query<&mut crate::gameplay_abilities::ActiveGameplayAbility>,
    mut activation_queue: ResMut<AbilityActivationQueue>,
    mut effect_queue: ResMut<GameplayEffectApplicationQueue>,
) {
    for (task_entity, mut task) in task_query.iter_mut() {
        if !task.tick() {
            continue;
        }

        match task.get_on_finished().clone() {
            AbilityTaskOnFinished::None => {}
            AbilityTaskOnFinished::EndAbility => {
                if let Ok(mut active_ability) =
                    active_ability_query.get_mut(task.get_active_ability())
                {
                    active_ability.set_status(AbilityActivationStatus::Ending);
                }
            }
            AbilityTaskOnFinished::ActivateAbility {
                source,
                target,
                handle,
            } => {
                activation_queue.push_activation(source, target, handle);
            }
            AbilityTaskOnFinished::ApplyGameplayEffect {
                source,
                target,
                effect,
                level,
            } => {
                let payload = EffectPayload::new(source, None, level);
                effect_queue.push_application(target, effect, payload);
            }
        }

        commands.entity(task_entity).despawn();
    }
}
