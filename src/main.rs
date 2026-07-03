use bevy::prelude::*;
use bevy_tools::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayAbilitySystemPlugin)
        .add_systems(Startup, register_initial_tags)
        .run();
}
fn register_initial_tags(mut register: GameplayTagRegister) {
    info!("--- Registering gameplay tags ---");

    // Register root tags
    let ability_tag = match register.request_or_register_tag("Ability") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };
    let _effect_tag = match register.request_or_register_tag("Effect") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };
    let _character_tag = match register.request_or_register_tag("Character") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };

    // Register child tags (parents are auto-registered)
    let _ability_fireball = match register.request_or_register_tag("Ability.Fireball") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };
    let _ability_heal = match register.request_or_register_tag("Ability.Heal") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };

    // Register multi-level tags
    let effect_debuff_stun = match register.request_or_register_tag("Effect.Debuff.Stun") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };
    let _effect_buff_speed = match register.request_or_register_tag("Effect.Buff.Speed") {
        Ok(tag) => tag,
        Err(err) => {
            error!("Failed to register tag: {err}");
            return;
        }
    };

    info!("Tag registration complete.");
    info!("Ability index: {}", ability_tag.get_bit_index_usize());
    info!(
        "Effect.Debuff.Stun index: {}",
        effect_debuff_stun.get_bit_index_usize()
    );
}
