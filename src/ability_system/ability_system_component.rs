// use crate::EffectContext;
// use crate::attributes::AttributeSet;
// use crate::gameplay_abilities::GameplayAbility;
use crate::gameplay_effects::{ActiveEffectHandle, ActiveGameplayEffect};
// use crate::gameplay_tags::{GameplayTagContainer, GameplayTagManager};
use bevy::prelude::*;
// use std::sync::Arc;

#[derive(Component, Default)]
pub struct AbilitySystemComponent {
    active_effects: Vec<ActiveGameplayEffect>,
}

impl AbilitySystemComponent {
    pub fn get_active_effects(&self) -> &[ActiveGameplayEffect] {
        &self.active_effects
    }

    pub fn add_active_effect(&mut self, effect: ActiveGameplayEffect) {
        self.active_effects.push(effect);
    }

    pub fn remove_active_effects(&mut self, handle_list: &[ActiveEffectHandle]) {
        self.active_effects
            .retain_mut(|effect| !handle_list.contains(&effect.get_handle()));
    }
}

// pub fn try_activate_ability(
//     source_entity: Entity,
//     target_entity: Entity,
//     ability: &Arc<GameplayAbility>,
//     attr_query: &mut Query<&mut AttributeSet>,
//     asc_query: &mut Query<&mut AbilitySystemComponent>,
//     tag_container_query: &mut Query<&mut GameplayTagContainer>,
//     tag_manager: &Res<GameplayTagManager>,
//     time: &Res<Time>,
// ) -> bool {
//     let source_tags = tag_container_query.get(source_entity).ok();

//     if let Some(tags) = source_tags {
//         let ability_tags = ability.get_tags();
//         if tags.has_any(ability_tags.get_activation_blocked_tags()) {
//             return false;
//         }
//         if !tags.has_all(ability_tags.get_activation_required_tags()) {
//             return false;
//         }
//     }

// let effect_context = EffectContext {
//     source: Some(source_entity),
//     target: Some(target_entity),
//     attr_set_query: ,
//     tag_container_query: tag_container_query,
//     ability_system_component_query: asc_query,
//     level: 1,
// };

// if let Some(cooldown_def) = &ability.cooldown {
//     if let Some(tags) = source_tags {
//         // 这里简化处理：假设 Cooldown Effect 的 Granted Tags 里包含了冷却 Tag
//         // 严谨的做法是去 Cooldown Effect 定义里找
//         if tags.has_any(cooldown_def.get_tags().get_granted_tags()) {
//             return false; // 还在冷却中
//         }
//     }
// }

// // 1.3 检查消耗 (Cost)
// // 这一步比较麻烦，需要预计算 Cost Effect 看看属性够不够减
// if let Some(cost_def) = &ability.cost {
//     // 创建一个临时的 Spec 来计算数值
//     // 检查 AttributeSet 里的 Current Value 是否 >= Cost Value
//     // (代码略，需要去 AttributeSet 里查)
// }

// // --- 步骤 2: 提交消耗 (Commit) ---

// // 2.1 应用消耗 (Apply Cost)
// if let Some(cost_def) = &ability.cost {
//     apply_gameplay_effect(
//         cost_def,
//         source_entity,
//         source_entity, // 消耗是应用给自己的
//         attr_query,
//         asc_query,
//         tag_container_query,
//         tag_manager,
//         handle_gen,
//         time,
//         1,
//     );
// }

// // 2.2 应用冷却 (Apply Cooldown)
// if let Some(cooldown_def) = &ability.cooldown {
//     apply_gameplay_effect(
//         cooldown_def,
//         source_entity,
//         source_entity, // 冷却也是应用给自己的
//         attr_query,
//         asc_query,
//         tag_container_query,
//         tag_manager,
//         handle_gen,
//         time,
//         1,
//     );
// }

// // --- 步骤 3: 执行技能逻辑 (Activate) ---

// println!("Ability {} Activated!", ability.name);

// // 3.1 应用技能效果 (Apply Effect)
// if let Some(effect_def) = &ability.effect {
//     apply_gameplay_effect(
//         effect_def,
//         source_entity,
//         target_entity, // 效果应用给目标
//         attr_query,
//         asc_query,
//         tag_container_query,
//         tag_manager,
//         handle_gen,
//         time,
//         1,
//     );
// }

// // 3.2 播放动画、生成投射物等
// // 在 Bevy 中，这里通常会发送一个 Event，让其他 System 去处理表现
// // commands.send_event(AbilityActivatedEvent { ... });
//
//     true
// }
