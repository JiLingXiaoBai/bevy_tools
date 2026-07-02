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

pub struct GameplayTagBundlePlugin;

impl PluginGroup for GameplayTagBundlePlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(UniqueNamePlugin)
            .add(GameplayTagPlugin)
            .add(RandomPlugin)
    }
}
