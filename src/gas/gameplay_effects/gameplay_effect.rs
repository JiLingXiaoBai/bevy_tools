use super::gameplay_effect_spec::{
    EffectDurationTicksSpec, EffectPeriodTicksSpec, GameplayEffectSpec,
};
use crate::ability_system::AbilitySystemComponent;
use crate::attributes::{AttributeSet, AttributeSetSnapshot};
use crate::gameplay_tags::{GameplayTag, GameplayTagContainer, GameplayTagManager};
use crate::modifiers::{Modifier, ModifierMagnitude, ModifierOperation};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Query;
use bevy::prelude::Res;
use std::sync::Arc;

pub struct EffectContext<'w, 's> {
    pub target: Option<Entity>,
    pub payload: &'w EffectPayload,
    pub attr_set_query: &'w Query<'w, 's, &'static AttributeSet>,
    pub tag_container_query: &'w Query<'w, 's, &'static GameplayTagContainer>,
    pub asc_query: &'w Query<'w, 's, &'static AbilitySystemComponent>,
}

impl<'w, 's> EffectContext<'w, 's> {
    pub fn source(&self) -> Entity {
        self.payload.get_source()
    }

    pub fn causer(&self) -> Option<Entity> {
        self.payload.get_causer()
    }

    pub fn level(&self) -> u32 {
        self.payload.get_level()
    }

    pub fn source_snapshot(&self) -> Option<&AttributeSetSnapshot> {
        self.payload.get_source_snapshot()
    }
}

#[derive(Clone)]
pub struct EffectPayload {
    source: Entity,
    causer: Option<Entity>,
    level: u32,
    source_snapshot: Option<AttributeSetSnapshot>,
}

impl EffectPayload {
    pub fn new(source: Entity, causer: Option<Entity>, level: u32) -> Self {
        Self {
            source,
            causer,
            level,
            source_snapshot: None,
        }
    }

    pub fn with_source_snapshot(mut self, source_snapshot: AttributeSetSnapshot) -> Self {
        self.source_snapshot = Some(source_snapshot);
        self
    }

    pub fn get_source(&self) -> Entity {
        self.source
    }

    pub fn get_causer(&self) -> Option<Entity> {
        self.causer
    }

    pub fn get_level(&self) -> u32 {
        self.level
    }

    pub fn get_source_snapshot(&self) -> Option<&AttributeSetSnapshot> {
        self.source_snapshot.as_ref()
    }
}

pub enum EffectDurationTicks {
    Instant,
    DurationTicks(ModifierMagnitude),
    Infinite,
}

impl EffectDurationTicks {
    pub fn make_spec(&self, context: &EffectContext) -> EffectDurationTicksSpec {
        match self {
            EffectDurationTicks::Instant => EffectDurationTicksSpec::Instant,
            EffectDurationTicks::DurationTicks(mm) => {
                EffectDurationTicksSpec::DurationTicks(magnitude_to_ticks(match mm {
                    ModifierMagnitude::Flat(f) => *f,
                    ModifierMagnitude::Calculated(mmc) => mmc.calculate(context),
                }))
            }
            EffectDurationTicks::Infinite => EffectDurationTicksSpec::Infinite,
        }
    }
}

pub struct EffectPeriodTicks {
    period_ticks: ModifierMagnitude,
    execute_on_applied: bool,
}

impl EffectPeriodTicks {
    pub fn new(period_ticks: ModifierMagnitude, execute_on_applied: bool) -> Self {
        Self {
            period_ticks,
            execute_on_applied,
        }
    }

    pub fn make_spec(&self, context: &EffectContext) -> EffectPeriodTicksSpec {
        let final_value = match &self.period_ticks {
            ModifierMagnitude::Flat(f) => *f,
            ModifierMagnitude::Calculated(mmc) => mmc.calculate(context),
        };
        let final_value = magnitude_to_ticks(final_value);
        EffectPeriodTicksSpec::new(final_value, self.execute_on_applied)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StackingType {
    None,
    AggregateBySource,
    AggregateByTarget,
}

#[derive(Default)]
pub struct TagRequirements {
    require_all: Vec<GameplayTag>,
    ignore_any: Vec<GameplayTag>,
}

impl TagRequirements {
    pub fn new(require_all: Vec<GameplayTag>, ignore_any: Vec<GameplayTag>) -> Self {
        Self {
            require_all,
            ignore_any,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.require_all.is_empty() && self.ignore_any.is_empty()
    }

    pub fn passes(&self, tags: Option<&GameplayTagContainer>) -> bool {
        if self.is_empty() {
            return true;
        }

        let Some(tags) = tags else {
            return false;
        };

        tags.has_all(&self.require_all) && !tags.has_any(&self.ignore_any)
    }

    pub fn passes_tag_slice(
        &self,
        tags: &[GameplayTag],
        tag_manager: &Res<GameplayTagManager>,
    ) -> bool {
        if self.is_empty() {
            return true;
        }

        let mut container = GameplayTagContainer::default();
        container.add_tags(tags, tag_manager);
        self.passes(Some(&container))
    }

    pub fn get_required_tags(&self) -> &[GameplayTag] {
        &self.require_all
    }

    pub fn get_ignored_tags(&self) -> &[GameplayTag] {
        &self.ignore_any
    }
}

#[derive(Default)]
pub struct GameplayEffectImmunityQuery {
    source_tags: TagRequirements,
    effect_tags: TagRequirements,
}

impl GameplayEffectImmunityQuery {
    pub fn new(source_tags: TagRequirements, effect_tags: TagRequirements) -> Self {
        Self {
            source_tags,
            effect_tags,
        }
    }

    pub fn matches(
        &self,
        source_tags: Option<&GameplayTagContainer>,
        effect_asset_tags: &[GameplayTag],
        tag_manager: &Res<GameplayTagManager>,
    ) -> bool {
        self.source_tags.passes(source_tags)
            && self
                .effect_tags
                .passes_tag_slice(effect_asset_tags, tag_manager)
    }
}

pub struct EffectTags {
    asset_tags: Vec<GameplayTag>,
    granted_tags: Vec<GameplayTag>,
    source_application_tags: TagRequirements,
    target_application_tags: TagRequirements,
    source_ongoing_tags: TagRequirements,
    target_ongoing_tags: TagRequirements,
    source_removal_tags: TagRequirements,
    target_removal_tags: TagRequirements,
    granted_application_immunity: Vec<GameplayEffectImmunityQuery>,
    remove_effects_with_tags: Vec<GameplayTag>,
}

impl EffectTags {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset_tags: Vec<GameplayTag>,
        granted_tags: Vec<GameplayTag>,
        source_application_tags: TagRequirements,
        target_application_tags: TagRequirements,
        source_ongoing_tags: TagRequirements,
        target_ongoing_tags: TagRequirements,
        source_removal_tags: TagRequirements,
        target_removal_tags: TagRequirements,
        granted_application_immunity: Vec<GameplayEffectImmunityQuery>,
        remove_effects_with_tags: Vec<GameplayTag>,
    ) -> Self {
        Self {
            asset_tags,
            granted_tags,
            source_application_tags,
            target_application_tags,
            source_ongoing_tags,
            target_ongoing_tags,
            source_removal_tags,
            target_removal_tags,
            granted_application_immunity,
            remove_effects_with_tags,
        }
    }

    pub fn get_asset_tags(&self) -> &[GameplayTag] {
        &self.asset_tags
    }

    pub fn get_granted_tags(&self) -> &[GameplayTag] {
        &self.granted_tags
    }

    pub fn get_required_tags(&self) -> &[GameplayTag] {
        self.target_application_tags.get_required_tags()
    }

    pub fn get_blocked_tags(&self) -> &[GameplayTag] {
        self.target_application_tags.get_ignored_tags()
    }

    pub fn get_source_application_tags(&self) -> &TagRequirements {
        &self.source_application_tags
    }

    pub fn get_target_application_tags(&self) -> &TagRequirements {
        &self.target_application_tags
    }

    pub fn get_source_ongoing_tags(&self) -> &TagRequirements {
        &self.source_ongoing_tags
    }

    pub fn get_target_ongoing_tags(&self) -> &TagRequirements {
        &self.target_ongoing_tags
    }

    pub fn get_source_removal_tags(&self) -> &TagRequirements {
        &self.source_removal_tags
    }

    pub fn get_target_removal_tags(&self) -> &TagRequirements {
        &self.target_removal_tags
    }

    pub fn get_granted_application_immunity(&self) -> &[GameplayEffectImmunityQuery] {
        &self.granted_application_immunity
    }

    pub fn get_remove_effects_with_tags(&self) -> &[GameplayTag] {
        &self.remove_effects_with_tags
    }
}

// stored as a Resource
pub struct GameplayEffect {
    modifiers: Vec<Modifier>,
    duration: EffectDurationTicks,
    period: Option<EffectPeriodTicks>,
    probability_to_apply: f64,
    stacking_type: StackingType,
    stacking_limit: u32,
    tags: EffectTags,
}

impl GameplayEffect {
    pub fn new(
        modifiers: Vec<Modifier>,
        duration: EffectDurationTicks,
        period: Option<EffectPeriodTicks>,
        probability_to_apply: f64,
        stacking_type: StackingType,
        stacking_limit: u32,
        tags: EffectTags,
    ) -> Self {
        Self {
            modifiers,
            duration,
            period,
            probability_to_apply,
            stacking_type,
            stacking_limit,
            tags,
        }
    }

    pub fn make_spec(self: &Arc<Self>, context: &EffectContext) -> GameplayEffectSpec {
        GameplayEffectSpec::new(
            self.clone(),
            self.modifiers
                .iter()
                .map(|m| m.make_spec(context))
                .collect(),
            self.duration.make_spec(context),
            self.period.as_ref().map(|p| p.make_spec(context)),
            self.stacking_type,
            self.stacking_limit,
        )
    }

    pub fn get_tags(&self) -> &EffectTags {
        &self.tags
    }

    pub fn has_only_add_modifiers(&self) -> bool {
        self.modifiers
            .iter()
            .all(|modifier| modifier.get_operation() == ModifierOperation::Add)
    }

    pub fn get_probability_to_apply(&self) -> f64 {
        self.probability_to_apply
    }
}

fn magnitude_to_ticks(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        0
    } else if value >= u32::MAX as f64 {
        u32::MAX
    } else {
        value.ceil() as u32
    }
}
