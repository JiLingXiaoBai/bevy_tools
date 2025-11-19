mod gameplay_tags;
mod unique_name;

use bevy::prelude::*;
use unique_name::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UniqueNamePool::default())
        .add_systems(Startup, setup_system)
        .add_systems(Update, (print_names_system, add_names_system))
        .run();
}
fn setup_system(mut pool: ResMut<UniqueNamePool>) {
    pool.clear();
    let _ = UniqueName::new("Player", &mut pool);
    let _ = UniqueName::new("Enemy", &mut pool);
}

fn print_names_system(mut pool: ResMut<UniqueNamePool>) {
    let player = UniqueName::new("Player", &mut pool);
    let enemy = UniqueName::new("Enemy", &mut pool);
    let player_name = player.as_str(&pool);
    let enemy_name = enemy.as_str(&pool);

    info!("Player: {}", player_name);
    info!("Enemy: {}", enemy_name);
}

fn add_names_system(mut pool: ResMut<UniqueNamePool>) {
    let _ = UniqueName::new("NPC", &mut pool);
}
