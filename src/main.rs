extern crate core;

mod gameplay_tags;
mod unique_name;

use bevy::prelude::*;
use unique_name::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UniqueNamePool::default())
        .add_systems(Startup, (setup_system, setup_names_system.after(setup_system)))
        .add_systems(Update, (print_names_system, add_names_system))
        .run();
}
fn setup_names_system(mut pool: ResMut<UniqueNamePool>) {
    info!("--- Initializing Unique Names ---");

    let player = pool.new_name("PlayerCharacter");
    info!("'PlayerCharacter' created: {:?}", player); // Output: UniqueName(1)

    let enemy = pool.new_name("EnemyUnit");
    info!("'EnemyUnit' created: {:?}", enemy); // Output: UniqueName(2)

    let player_again = pool.new_name("PlayerCharacter");
    info!("'PlayerCharacter' retrieved: {:?}", player_again); // Output: UniqueName(1)

    let empty_name = pool.new_name("");
    info!("Empty name created: {:?}", empty_name); // Output: UniqueName(0)

    assert_eq!(player, player_again);
}

fn setup_system(mut pool: ResMut<UniqueNamePool>) {
    pool.clear();
}

fn print_names_system(mut pool: ResMut<UniqueNamePool>) {
    let player = pool.new_name("Player");
    let enemy = pool.new_name("Enemy");
    let player_name = pool.get_display_str(&player);
    let enemy_name = pool.get_display_str(&enemy);

    info!("Player: {}", player_name);
    info!("Enemy: {}", enemy_name);
}

fn add_names_system(mut pool: ResMut<UniqueNamePool>) {
    let _ = pool.new_name("NPC");
}
