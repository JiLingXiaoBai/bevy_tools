mod gas;
mod randoms;
mod unique_names;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
pub use gas::*;
pub use randoms::*;
pub use unique_names::*;
extern crate core;

pub struct GameplayTagPlugin;

impl Plugin for GameplayTagPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayTagManager>();
    }
}

pub struct UniqueNamePlugin;

impl Plugin for UniqueNamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UniqueNamePool>();
    }
}

pub struct RandomPlugin;

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Random>();
    }
}

pub struct GameplayAbilitySystemRuntimePlugin;

impl Plugin for GameplayAbilitySystemRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttributeIdManager>()
            .init_resource::<AbilityActivationQueue>()
            .init_resource::<GameplayEffectApplicationQueue>()
            .add_systems(
                FixedUpdate,
                (
                    update_active_effect_tag_requirements_system,
                    (tick_effect_duration_system, tick_effect_period_system),
                    tick_ability_tasks_system,
                    process_gameplay_effect_application_queue_system,
                    process_ability_activation_queue_system,
                    cleanup_finished_abilities_system,
                    recalculate_attribute_sets_system,
                )
                    .chain(),
            );
    }
}

pub struct GameplayAbilitySystemPlugin;

impl PluginGroup for GameplayAbilitySystemPlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(UniqueNamePlugin)
            .add(GameplayTagPlugin)
            .add(RandomPlugin)
            .add(GameplayAbilitySystemRuntimePlugin)
    }
}
