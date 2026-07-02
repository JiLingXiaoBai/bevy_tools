use super::*;
use crate::gameplay_effects::ActiveEffectHandle;
use crate::modifiers::ModifierSpec;
use crate::settings::GameplayAbilitySystemSettings;
use bevy::prelude::*;

pub const ATTRIBUTE_SET_SIZE: usize = GameplayAbilitySystemSettings::ATTRIBUTE_SET_SIZE;

pub type AttributePostExecute = fn(&mut AttributeSet, AttributeId, f64, f64);

#[derive(Component)]
pub struct AttributeSet {
    attributes: Box<[Option<Box<Attribute>>]>,
    post_execute: Option<AttributePostExecute>,
}

impl Default for AttributeSet {
    fn default() -> Self {
        let mut attrs = Vec::with_capacity(ATTRIBUTE_SET_SIZE);
        for _ in 0..ATTRIBUTE_SET_SIZE {
            attrs.push(None);
        }
        Self {
            attributes: attrs.into_boxed_slice(),
            post_execute: None,
        }
    }
}

impl AttributeSet {
    pub fn initialize_attribute(
        &mut self,
        id: AttributeId,
        base_value: f64,
        executor: Option<fn(&Aggregator, f64) -> f64>,
    ) {
        self.initialize_attribute_with_clamp(id, base_value, executor, AttributeClamp::None);
    }

    pub fn initialize_attribute_with_clamp(
        &mut self,
        id: AttributeId,
        base_value: f64,
        executor: Option<fn(&Aggregator, f64) -> f64>,
        clamp: AttributeClamp,
    ) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        let mut attr = Box::new(Attribute::default());
        attr.init_with_clamp(base_value, executor, clamp);
        self.attributes[index] = Some(attr);
        self.recalculate_all();
    }

    pub fn set_attribute_clamp(&mut self, id: AttributeId, clamp: AttributeClamp) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &mut self.attributes[index] {
            attr.set_clamp(clamp);
        }
        self.recalculate_all();
    }

    pub fn set_post_execute(&mut self, post_execute: Option<AttributePostExecute>) {
        self.post_execute = post_execute;
    }

    pub fn recalculate_attribute(&mut self, id: AttributeId) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        self.recalculate_all();
    }

    pub fn recalculate_all(&mut self) {
        for attr in self.attributes.iter_mut().flatten() {
            attr.recalculate();
        }

        for attr in self.attributes.iter_mut().flatten() {
            let clamp = attr.get_clamp();
            let (min, max) = resolve_static_clamp_bounds(clamp);
            attr.clamp_current(min, max);
        }

        let current_values = self
            .attributes
            .iter()
            .map(|attr| attr.as_ref().map(|attr| attr.get_current_value()))
            .collect::<Vec<_>>();

        for attr in self.attributes.iter_mut().flatten() {
            let clamp = attr.get_clamp();
            let (min, max) = resolve_clamp_bounds(clamp, &current_values);
            attr.clamp_current(min, max);
        }
    }

    pub fn get_current_value(&self, id: AttributeId) -> Option<f64> {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &self.attributes[index] {
            return Some(attr.get_current_value());
        }
        None
    }

    pub fn apply_instant_modifier(&mut self, spec: &ModifierSpec) {
        let index = spec.get_id().to_index();
        debug_assert!(index < self.attributes.len());
        self.recalculate_all();
        let old_value = self.get_current_value(spec.get_id());
        if let Some(attr) = &mut self.attributes[index] {
            attr.modify_base_value(spec);
        }
        self.recalculate_all();

        if let (Some(old_value), Some(new_value), Some(post_execute)) = (
            old_value,
            self.get_current_value(spec.get_id()),
            self.post_execute,
        ) {
            post_execute(self, spec.get_id(), old_value, new_value);
        }
    }

    pub fn apply_duration_modifier(&mut self, spec: &ModifierSpec, handle: ActiveEffectHandle) {
        let index = spec.get_id().to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &mut self.attributes[index] {
            attr.apply_modifier_spec(spec, handle);
        }
    }

    pub fn remove_modifiers(&mut self, handle: ActiveEffectHandle) {
        self.attributes
            .iter_mut()
            .flatten()
            .for_each(|attr| attr.remove_modifier_by_handle(handle));
    }

    pub fn make_snapshot(&self, source_entity: Entity) -> AttributeSetSnapshot {
        let mut new_attrs = Vec::with_capacity(self.attributes.len());
        for attr_opt in self.attributes.iter() {
            if let Some(attr) = attr_opt {
                new_attrs.push(Some(Box::new(attr.make_snapshot())));
            } else {
                new_attrs.push(None);
            }
        }

        AttributeSetSnapshot::new(new_attrs, source_entity)
    }
}

fn resolve_clamp_bounds(
    clamp: AttributeClamp,
    current_values: &[Option<f64>],
) -> (Option<f64>, Option<f64>) {
    let AttributeClamp::Range { min, max } = clamp else {
        return (None, None);
    };

    (
        resolve_clamp_bound(min, current_values),
        resolve_clamp_bound(max, current_values),
    )
}

fn resolve_static_clamp_bounds(clamp: AttributeClamp) -> (Option<f64>, Option<f64>) {
    let AttributeClamp::Range { min, max } = clamp else {
        return (None, None);
    };

    (
        resolve_static_clamp_bound(min),
        resolve_static_clamp_bound(max),
    )
}

fn resolve_static_clamp_bound(bound: Option<AttributeClampBound>) -> Option<f64> {
    match bound {
        Some(AttributeClampBound::Static(value)) => Some(value),
        Some(AttributeClampBound::Attribute(_)) | None => None,
    }
}

fn resolve_clamp_bound(
    bound: Option<AttributeClampBound>,
    current_values: &[Option<f64>],
) -> Option<f64> {
    match bound {
        Some(AttributeClampBound::Static(value)) => Some(value),
        Some(AttributeClampBound::Attribute(id)) => {
            current_values.get(id.to_index()).copied().flatten()
        }
        None => None,
    }
}

pub fn recalculate_attribute_sets_system(mut query: Query<&mut AttributeSet>) {
    for mut attr_set in query.iter_mut() {
        attr_set.recalculate_all();
    }
}
