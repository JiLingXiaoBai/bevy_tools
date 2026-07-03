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
    dirty: bool,
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
            dirty: true,
        }
    }
}

impl AttributeSet {
    pub fn initialize_attribute(
        &mut self,
        id: AttributeId,
        base_value: f64,
        executor: Option<fn(&Aggregator, f64) -> f64>,
        clamp: AttributeClamp,
    ) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        let mut attr = Box::new(Attribute::default());
        attr.init(base_value, executor, clamp);
        self.attributes[index] = Some(attr);
        self.mark_dirty();
    }

    pub fn set_attribute_clamp(&mut self, id: AttributeId, clamp: AttributeClamp) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &mut self.attributes[index] {
            attr.set_clamp(clamp);
            self.mark_dirty();
        }
    }

    pub fn set_post_execute(&mut self, post_execute: Option<AttributePostExecute>) {
        self.post_execute = post_execute;
    }

    pub fn recalculate_attribute(&mut self, id: AttributeId) {
        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if self.attributes[index].is_some() {
            self.mark_dirty();
        }
        self.recalculate_all();
    }

    pub fn recalculate_all(&mut self) {
        if !self.dirty {
            return;
        }

        for attr in self.attributes.iter_mut().flatten() {
            attr.recalculate();
        }

        self.dirty = false;
    }

    pub fn get_current_value(&mut self, id: AttributeId) -> Option<f64> {
        self.recalculate_all();

        let index = id.to_index();
        debug_assert!(index < self.attributes.len());
        if let Some(attr) = &mut self.attributes[index] {
            return Some(attr.get_current_value());
        }
        None
    }

    pub fn apply_instant_modifier(&mut self, spec: &ModifierSpec) {
        let index = spec.get_id().to_index();
        debug_assert!(index < self.attributes.len());
        let old_value = self.get_current_value(spec.get_id());
        if let Some(attr) = &mut self.attributes[index] {
            attr.modify_base_value(spec);
            self.mark_dirty();
        }

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
            self.mark_dirty();
        }
    }

    pub fn remove_modifiers(&mut self, handle: ActiveEffectHandle) {
        let mut removed_from_any = false;
        for attr in self.attributes.iter_mut().flatten() {
            attr.remove_modifier_by_handle(handle);
            removed_from_any = true;
        }
        if removed_from_any {
            self.mark_dirty();
        }
    }

    pub fn make_snapshot(&mut self, source_entity: Entity) -> AttributeSetSnapshot {
        self.recalculate_all();

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

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

pub fn recalculate_attribute_sets_system(mut query: Query<&mut AttributeSet>) {
    for mut attr_set in query.iter_mut() {
        attr_set.recalculate_all();
    }
}
