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

pub struct GameplayAbilitySystemPlugin;

impl Plugin for GameplayAbilitySystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttributeIdManager>().add_systems(
            FixedUpdate,
            (
                update_active_effect_tag_requirements_system,
                (tick_effect_duration_system, tick_effect_period_system),
                cleanup_finished_abilities_system,
                recalculate_attribute_sets_system,
            )
                .chain(),
        );
    }
}

pub struct GameplayTagBundlePlugin;

impl PluginGroup for GameplayTagBundlePlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(UniqueNamePlugin)
            .add(GameplayTagPlugin)
            .add(RandomPlugin)
            .add(GameplayAbilitySystemPlugin)
    }
}
