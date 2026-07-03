use crate::ability_system::AbilitySystemParams;
use crate::gameplay_effects::{GameplayEffect, apply_gameplay_effect};
use crate::settings::GameplayAbilitySystemSettings;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone)]
pub struct GameplayEffectApplicationRequest {
    source: Entity,
    target: Entity,
    effect: Arc<GameplayEffect>,
    level: u32,
}

impl GameplayEffectApplicationRequest {
    pub fn new(source: Entity, target: Entity, effect: Arc<GameplayEffect>, level: u32) -> Self {
        Self {
            source,
            target,
            effect,
            level,
        }
    }

    pub fn get_source(&self) -> Entity {
        self.source
    }

    pub fn get_target(&self) -> Entity {
        self.target
    }

    pub fn get_effect(&self) -> &Arc<GameplayEffect> {
        &self.effect
    }

    pub fn get_level(&self) -> u32 {
        self.level
    }
}

#[derive(Resource)]
pub struct GameplayEffectApplicationQueue {
    requests: VecDeque<GameplayEffectApplicationRequest>,
    max_applications_per_tick: usize,
}

impl Default for GameplayEffectApplicationQueue {
    fn default() -> Self {
        Self {
            requests: VecDeque::new(),
            max_applications_per_tick:
                GameplayAbilitySystemSettings::GAMEPLAY_EFFECT_APPLICATION_QUEUE_MAX_PER_TICK,
        }
    }
}

impl GameplayEffectApplicationQueue {
    pub fn push(&mut self, request: GameplayEffectApplicationRequest) {
        self.requests.push_back(request);
    }

    pub fn push_application(
        &mut self,
        source: Entity,
        target: Entity,
        effect: Arc<GameplayEffect>,
        level: u32,
    ) {
        self.push(GameplayEffectApplicationRequest::new(
            source, target, effect, level,
        ));
    }

    pub fn pop(&mut self) -> Option<GameplayEffectApplicationRequest> {
        self.requests.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn clear(&mut self) {
        self.requests.clear();
    }

    pub fn max_applications_per_tick(&self) -> usize {
        self.max_applications_per_tick
    }

    pub fn set_max_applications_per_tick(&mut self, max_applications_per_tick: usize) {
        self.max_applications_per_tick = max_applications_per_tick.max(1);
    }
}

pub fn process_gameplay_effect_application_queue_system(
    mut effect_queue: ResMut<GameplayEffectApplicationQueue>,
    mut params: AbilitySystemParams,
) {
    let max_applications = effect_queue.max_applications_per_tick();
    for _ in 0..max_applications {
        let Some(request) = effect_queue.pop() else {
            return;
        };

        apply_gameplay_effect(
            request.get_source(),
            request.get_target(),
            request.get_effect(),
            &mut params,
            request.get_level(),
        );
    }
}
