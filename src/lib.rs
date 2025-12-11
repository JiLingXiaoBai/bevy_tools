mod gameplay_tags;
mod unique_name;
mod attribute;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
pub use gameplay_tags::*;
pub use unique_name::*;
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
pub struct GameplayTagBundlePlugin;

impl PluginGroup for GameplayTagBundlePlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(UniqueNamePlugin)
            .add(GameplayTagPlugin)
    }
}
