use bevy::prelude::*;
use bevy_tools::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayTagBundlePlugin)
        .add_systems(Startup, register_initial_tags)
        .run();
}
fn register_initial_tags(mut register: GameplayTagRegister) {
    info!("--- 正在注册游戏性标签 ---");

    // 注册根标签
    let ability_tag = register.request_or_register_tag("Ability");
    let _effect_tag = register.request_or_register_tag("Effect");
    let _character_tag = register.request_or_register_tag("Character");

    // 注册子标签 (会自动注册其父标签)
    let _ability_fireball = register.request_or_register_tag("Ability.Fireball");
    let _ability_heal = register.request_or_register_tag("Ability.Heal");

    // 注册多级标签
    let effect_debuff_stun = register.request_or_register_tag("Effect.Debuff.Stun");
    let _effect_buff_speed = register.request_or_register_tag("Effect.Buff.Speed");

    info!("标签注册完成。");
    info!("Ability 索引: {}", ability_tag.get_bit_index_u16());
    info!(
        "Effect.Debuff.Stun 索引: {}",
        effect_debuff_stun.get_bit_index_u16()
    );
}
