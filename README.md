# bevy_tools

`bevy_tools` 是一个基于 Bevy ECS 的轻量 Gameplay Ability System 实验项目。当前重点是搭建一套可组合的战斗/技能运行时，包括 GameplayTag、Attribute、GameplayEffect、GameplayAbility、AbilityTask 和 AbilitySystemComponent。

项目依赖：

- Rust edition 2024
- Bevy `0.19.0`
- rand `0.10.1`

## 核心目标

这个项目试图把常见 GAS 流程拆成几个清晰层级：

- `GameplayTag`：描述状态、分类、阻挡、免疫、需求等标签语义
- `AttributeSet`：保存生命、攻击力、资源等属性，并支持 modifier 聚合
- `GameplayEffect`：描述属性修改、持续时间、周期效果、堆叠、标签授予和免疫
- `GameplayAbility`：描述技能定义、消耗、冷却、启动任务和标签规则
- `AbilityTask`：承担技能执行行为，例如等待、触发事件、应用 effect
- `AbilitySystemComponent`：挂在角色实体上，管理技能授予、激活和生命周期

项目整体思路是：Ability 负责启动和组织行为，Effect 负责真正修改属性或授予状态，Task 负责把行为拆成可 tick、可取消、可扩展的执行单元。

## 目录结构

```text
src/
  lib.rs
  main.rs
  gas.rs
  randoms.rs
  unique_names.rs
  gas/
    ability_system.rs
    attributes.rs
    gameplay_abilities.rs
    gameplay_effects.rs
    gameplay_tags.rs
    modifiers.rs
    settings.rs
    ability_system/
      ability_activation_queue.rs
      ability_system_component.rs
    attributes/
      attribute.rs
      attribute_aggregator.rs
      attribute_id_manager.rs
      attribute_set.rs
      attribute_set_snapshot.rs
      attribute_snapshot.rs
    gameplay_abilities/
      ability_task.rs
      active_gameplay_ability.rs
      gameplay_ability.rs
      gameplay_ability_spec.rs
    gameplay_effects/
      active_gameplay_effect.rs
      gameplay_effect.rs
      gameplay_effect_application_queue.rs
      gameplay_effect_spec.rs
    gameplay_tags/
      gameplay_tag.rs
      gameplay_tag_container.rs
      gameplay_tag_manager.rs
    modifiers/
      modifier.rs
  randoms/
    random.rs
  unique_names/
    unique_name.rs
```

## 插件入口

项目主要入口在 `src/lib.rs`。

`GameplayAbilitySystemPlugin` 是插件组，会注册：

- `UniqueNamePlugin`
- `GameplayTagPlugin`
- `RandomPlugin`
- `GameplayAbilitySystemRuntimePlugin`

`GameplayAbilitySystemRuntimePlugin` 会初始化运行时资源：

- `AttributeIdManager`
- `AbilityActivationQueue`
- `GameplayEffectApplicationQueue`
- `ActiveGameplayEffectTargetIndex`

并在 `FixedUpdate` 中通过 `GameplayAbilitySystemSet` 声明运行阶段：

1. `UpdateEffectTagRequirements`
2. `EffectTicks`
3. `AbilityTasks`
4. `Queues`
5. `Cleanup`
6. `RecalculateAttributes`

系统不再整体串成一条大链，而是只声明必要依赖：

```text
ActiveEffect tag requirements 在 effect tick 前更新
Effect tick 和 AbilityTask 都在队列消费前完成
Effect 队列先于 Ability 队列消费
队列消费后清理结束的 ActiveAbility
最后只重算发生过变化的 AttributeSet
```

这样保留运行语义，同时允许 Bevy 调度器并行处理没有直接依赖的系统。

## GameplayTag

Tag 用于表达游戏状态、技能分类、伤害类型、冷却、免疫、需求和阻挡规则。

注册 tag 使用 `GameplayTagRegister`。当注册类似 `"Ability.Fireball.Explode"` 的层级 tag 时，系统会递归确保父级 tag 存在：

```text
Ability
Ability.Fireball
Ability.Fireball.Explode
```

每个 tag 会被分配一个 `u16` bit index。

`GameplayTagManager` 保存：

- tag 名到 index 的映射
- 父 tag index
- 子 tag 列表
- 每个 tag 的 inherited bits

`GameplayTagContainer` 是组件，内部使用 bitset 和引用计数：

- 添加 tag 时，会同时增加自身和所有父级 tag 的引用计数，并设置对应 bit
- 移除 tag 时，会减少自身和所有父级 tag 的引用计数，只有计数归零才清 bit
- `has_all` 和 `has_any` 通过 bitset 快速判断
- 上层也可以直接传入预计算 bitset，避免重复从 tag 列表构造查询 bitset

因此，如果实体拥有 `Damage.Fire`，查询 `Damage` 也会命中。这让父级分类判断很自然，例如所有元素伤害都能归入 `Damage`。

## Attribute

属性系统由 `AttributeSet` 组件保存。每个具体属性是一个 `Attribute`。

`Attribute` 内部字段：

- `base`：基础值
- `evaluated`：base 经过持续 modifier 聚合后的值
- `current`：经过 clamp 后的最终当前值
- `aggregator`：保存 duration modifier
- `dirty`：标记是否需要重新计算
- `clamp`：当前支持 `AttributeClamp::None` 和静态范围 `AttributeClamp::Range { min, max }`

### 属性初始化

通过下面方法初始化属性：

```rust
AttributeSet::initialize_attribute(id, base_value, executor, clamp)
```

`executor` 是可选的聚合计算函数。如果不传，默认聚合顺序是：

1. 如果存在 Override，直接返回 Override 值
2. Add modifier 累加
3. PercentAdd modifier 汇总后乘以 `1 + percent_sum`
4. Multiply modifier 逐个相乘

### 懒重算

当前属性采用懒重算策略：

- base 或 modifier 改动时，只标记 dirty
- `AttributeSet::get_current_value` 会先调用 `recalculate_all`
- `AttributeSet::make_snapshot` 会先调用 `recalculate_all`
- `Attribute::get_current_value` 内部也会调用 `recalculate`

`Attribute::recalculate` 中，只有 dirty 时才重新计算 `evaluated`，但每次都会执行 `clamp_current`。这样 clamp 逻辑完全留在 `attribute.rs` 中，`AttributeSet` 不需要知道具体 clamp 细节。

### Instant Modifier

Instant modifier 会直接修改 `base`。

流程：

```text
读取旧 current
  -> 修改 base
  -> 标记 dirty
  -> 读取新 current
  -> 如果存在 post_execute，则触发 post_execute
```

这适合伤害、治疗、资源消耗等一次性修改。

### Duration Modifier

Duration modifier 不直接改 base，而是进入 `Aggregator`。

每个 duration modifier 会带有 `ActiveEffectHandle`。当对应 `ActiveGameplayEffect` 被移除时，`AttributeSet::remove_modifiers(handle)` 会移除这个 handle 关联的所有 modifier。

运行时 cleanup 路径会优先使用 `remove_modifiers_for_attributes(handle, ids)`，只访问该 effect 实际修改过的属性，避免每次移除 modifier 都扫描完整属性表。

这适合 buff、debuff、装备加成、临时护盾等持续影响。

## Modifier

Modifier 分为定义期和 spec 期。

定义期：

```rust
Modifier {
    id,
    op,
    magnitude,
}
```

`ModifierMagnitude` 支持：

- `Flat(f64)`
- `Calculated(Box<dyn ModifierMagnitudeCalculation>)`

`Calculated` 会通过 `EffectContext` 在创建 spec 时计算最终数值。

spec 期：

```rust
ModifierSpec {
    id,
    op,
    value,
}
```

Effect 在应用前会生成 `GameplayEffectSpec`，这一步会冻结 modifier 数值。也就是说，后续 source 属性变化不会影响已经创建好的 spec，除非上层选择在更晚时机重新创建 payload/spec。

## EffectPayload 与 EffectContext

外部应用 Effect 时传入 `EffectPayload`。

`EffectPayload` 保存：

- `source`
- `causer`
- `level`
- `source_snapshot`

`EffectContext` 是 make spec 时临时构造的上下文，内部引用：

- `payload`
- `attr_set_query`
- `tag_container_query`
- `asc_query`
- `target`

二者语义区分：

- `EffectPayload` 是可跨队列、跨系统传递的数据载体
- `EffectContext` 是 spec 计算期间的临时视图

如果伤害、元素加成、技能等级等数值应该以发射瞬间为准，可以在发射时创建 `AttributeSetSnapshot`，并通过 `EffectPayload::with_source_snapshot(...)` 放入 payload。

如果数值应该以命中瞬间为准，则不要放 snapshot，让 `ModifierMagnitudeCalculation` 通过 `EffectContext` 读取 source 当前属性。

## GameplayEffect

Effect 的执行被拆成四层：

```text
GameplayEffect 定义
  -> GameplayEffectSpec 冻结数值
  -> GameplayEffectApplicationPlan 预检查和计划
  -> execute_gameplay_effect_plan 真正执行
```

### GameplayEffect 定义

`GameplayEffect` 包含：

- modifiers
- duration
- period
- probability_to_apply
- stacking_policy
- tags

duration 支持：

- `Instant`
- `DurationTicks`
- `Infinite`

period 支持：

- period tick 数
- 是否 `execute_on_applied`

### EffectTags

Effect tag 规则包括：

- asset tags
- granted tags
- source application requirements
- target application requirements
- source ongoing requirements
- target ongoing requirements
- source removal requirements
- target removal requirements
- granted application immunity
- remove effects with tags

这些规则分别控制：

- 能不能应用
- 应用后授予什么 tag
- 持续期间是否被 inhibit
- 什么时候移除
- 是否免疫后续 effect
- 新 effect 应用时是否先移除旧 effect

### prepare_gameplay_effect

`prepare_gameplay_effect(target, effect_def, params, payload)` 会执行：

1. 根据 `probability_to_apply` 做随机判定
2. 检查 source 和 target 的 application tag requirements
3. 检查目标身上已有 active effect 提供的 application immunity
4. 构造 `EffectContext`
5. 调用 `effect_def.make_spec(&context)` 冻结 modifier、duration、period 和 stacking policy
6. 拒绝 `DurationTicks(0)`
7. 判断是否需要目标拥有 `AttributeSet`
8. 收集 `remove_effects_with_tags` 命中的旧 effect
9. 判断是否能与已有 active effect 堆叠
10. 返回 `GameplayEffectApplicationPlan`

### execute_gameplay_effect_plan

执行 plan 时先移除 `removed_effects`，再按类型执行：

- `Instant`：直接对目标 AttributeSet 应用 instant modifier
- `CreateActive`：创建 `ActiveGameplayEffect` 实体
- `StackExisting`：更新已有 active effect 的 stack count，并按 stacking policy 刷新 duration、period 和数值

## ActiveGameplayEffect 生命周期

`ActiveGameplayEffect` 是 ECS 实体组件，保存：

- `spec`
- `source`
- `target`
- `stack_count`
- `inhibited`

创建 active effect 时：

1. spawn `ActiveGameplayEffect`
2. 如果是 duration effect，插入 `ActiveEffectDurationTicks`
3. 如果是 periodic effect，插入 `ActiveEffectPeriodTicks`
4. 如果没有 period，则把 duration modifier 立刻加到目标 AttributeSet
5. 如果有 period 且 `execute_on_applied` 为 true，则立刻执行一次 instant modifier
6. effect entity 设置为 target 的子实体
7. 向 target 的 `GameplayTagContainer` 添加 granted tags

运行时还会把 effect handle 写入 `ActiveGameplayEffectTargetIndex`。后续按 target 查找可堆叠 effect、remove-with-tags、application immunity 时，会优先走 target 索引，而不是扫描所有 active effect。

索引同时保存 handle 到 target 的反查，并通过 `reconcile_active_effect_target_index_system` 监听 `RemovedComponents<ActiveGameplayEffect>` 做兜底清理，避免外部直接 despawn effect entity 后留下陈旧索引。

### Duration Tick

`tick_effect_duration_system` 每个 fixed tick 减少 `remain_ticks`。

归零时根据 `StackExpirationPolicy` 处理：

- `RemoveAllStacks`：清理整个 active effect
- `RemoveSingleStack`：如果 stack 大于 1，则掉一层、刷新 duration，并重算 duration modifier；否则清理整个 active effect

清理 active effect 时会：

- 从目标 AttributeSet 移除该 handle 对应的 duration modifier
- 从目标 GameplayTagContainer 移除 granted tags
- 从 `ActiveGameplayEffectTargetIndex` 移除该 handle
- despawn effect entity

### Period Tick

`tick_effect_period_system` 只处理未 inhibited 的 periodic effect。

当 `current_tick >= period_ticks` 时：

```text
current_tick 归零
  -> 对目标 AttributeSet 应用一次 instant modifier
```

周期伤害或周期治疗会修改 base，而不是进入 Aggregator。

### Ongoing 和 Removal Requirements

`update_active_effect_tag_requirements_system` 先判断 removal requirements：

- source removal tags 命中
- target removal tags 命中

命中则直接清理 effect。

如果未移除，再判断 ongoing requirements：

- 不满足且当前未 inhibited：移除 duration modifier 和 granted tags，标记 inhibited
- 满足且当前 inhibited：重新应用 duration modifier 和 granted tags，解除 inhibited

## StackingPolicy

堆叠规则由 `StackingPolicy` 统一描述，而不是分散硬编码。

字段：

- `stacking_type`
- `stack_limit`
- `magnitude_policy`
- `duration_policy`
- `period_policy`
- `overflow_policy`
- `expiration_policy`

### StackingType

- `None`：不堆叠
- `AggregateBySource`：同 target、同 effect definition、同 source 才堆叠
- `AggregateByTarget`：同 target、同 effect definition 即堆叠

### StackMagnitudePolicy

- `None`：modifier 不随层数变化
- `Linear`：modifier 数值乘以 stack count

### StackDurationPolicy

- `KeepExisting`：成功堆叠时保留原 duration
- `RefreshOnSuccessfulStack`：成功堆叠时刷新 duration

### StackPeriodPolicy

- `KeepCurrentTick`：成功堆叠时保留当前 period tick
- `ResetOnSuccessfulStack`：成功堆叠时 period tick 归零

### StackOverflowPolicy

- `RejectApplication`：达到上限后拒绝新应用
- `RefreshDuration`：达到上限后不增加层数，只刷新 duration

### StackExpirationPolicy

- `RemoveAllStacks`：duration 到期时移除整个 active effect
- `RemoveSingleStack`：duration 到期时掉一层，最后一层到期才移除 effect

便捷构造：

```rust
StackingPolicy::non_stacking()
StackingPolicy::linear_refreshing(stacking_type, stack_limit)
```

`linear_refreshing` 对应常见旧式堆叠行为：线性放大数值、成功堆叠刷新 duration、重置 period tick、达到上限后拒绝继续应用、到期移除全部层数。

## GameplayAbility

Ability 是技能定义数据。

`GameplayAbility` 包含：

- ability tags
- startup tasks
- cooldown effect
- cost effect
- activation effects
- `end_on_activation`
- `allow_multiple_instances`

实体通过 `AbilitySystemComponent` 持有技能。ASC 内部保存 `GameplayAbilitySpec`：

- handle
- ability definition
- level
- input id
- active count

ASC 同时维护 `AbilitySpecHandle -> Vec index` 的索引，按 handle 激活、清理和 active count 更新时不需要线性扫描整个 ability 列表。

## Ability 激活流程

激活有两个入口。

可以直接调用：

```text
try_activate_ability_by_handle(source, target, handle, params)
```

也可以进入队列：

```text
AbilityActivationQueue::push_activation(source, target, handle)
```

队列由 `process_ability_activation_queue_system` 在 `FixedUpdate` 中消费。

### try_activate_ability_by_handle

完整流程：

1. 从 source 的 ASC 中找到 ability spec
2. 检查 `allow_multiple_instances` 和 active count
3. 检查 ability tag、cooldown tag 等激活需求
4. prepare cost/cooldown commit plans，并用 cost plan 检查资源是否足够
5. 根据 ability 的 `cancel_abilities_with_tags` 取消其它 active ability
6. 调用 `start_ability`
7. 执行已准备好的 commit plans
8. commit 失败则 rollback
9. commit 成功后应用 activation effects
10. spawn startup tasks
11. 如果 `end_on_activation` 为 true，把 active ability 标记为 `Ending`

### start_ability

`start_ability` 会：

- 把 ability 的 `block_abilities_with_tags` 加到 ASC 内部 `blocked_ability_tags`
- 让对应 `GameplayAbilitySpec` 的 active count 加一
- spawn `ActiveGameplayAbility`
- 把 active ability entity 设置为 source 的子实体

### can_activate_ability

激活检查包括：

- ASC 内部 `blocked_ability_tags` 是否阻挡当前 ability asset tags
- source tags 是否满足 activation required tags
- source tags 是否命中 activation blocked tags
- cooldown granted tags 是否已存在
- cost 是否可支付

cost 检查目前只接受全部 modifier 都是 Add 的 cost effect。检查公式是：

```text
current_value + cost_value >= 0
```

### commit_ability

commit 会先 prepare cost plan 和 cooldown plan，两个都能准备成功才执行。

这样可以避免半提交状态，例如 cost 已扣除但 cooldown 应用失败。

`try_activate_ability_by_handle` 会复用同一份 cost plan 做资源检查和真正执行，避免 cost effect spec 在激活主路径中重复构建。

如果 commit 失败，会 rollback：

- active count 减一
- 移除 ASC 内部 blocked tags
- despawn active ability 及其子 task

## ActiveGameplayAbility

`ActiveGameplayAbility` 是运行期技能实例，保存：

- source
- spec handle
- target
- status

status 支持：

- `Activating`
- `Active`
- `Ending`
- `Cancelled`

`end_ability` 和 `cancel_ability` 不会立刻 despawn，而是把状态改为 `Ending` 或 `Cancelled`。

`cleanup_finished_abilities_system` 统一清理这些实例：

- active count 减一
- 移除 blocked tags
- `despawn_children().despawn()` 清理 active ability 和子 AbilityTask

如果 source ASC 已经不存在，也会直接清理 active ability 和子 task。

## AbilityTask

`GameplayAbility` 本身偏数据定义，实际执行行为通过 `startup_tasks` 生成 `AbilityTask` 实体。

当前定义层支持：

- `AbilityTaskDef::Instant`
- `AbilityTaskDef::WaitTicks`

task 完成后的定义层行为支持：

- `None`
- `EndAbility`
- `EmitEvent { event_id }`
- `ApplyGameplayEffectToTarget { effect }`

运行期 `AbilityTaskOnFinished` 额外支持 `ActivateAbility`，可以把其它 ability 激活请求推入队列。

### Task 创建

Ability 激活成功并完成 cost/cooldown 后，会调用：

```text
spawn_startup_ability_tasks(active_handle, source, target, spec_handle, level, startup_tasks)
```

每个 task 会设置为对应 `ActiveGameplayAbility` 的子实体。

### Task Tick

`tick_ability_tasks_system` 每个 fixed tick 处理所有 task：

1. 如果对应 active ability 不存在，task 自己 despawn
2. 如果 active ability 状态不是 `Active`，task 自己 despawn
3. tick task
4. 如果 task 未完成，继续等待
5. task 完成后执行 on_finished
6. task despawn

### Trigger / Observer

`EmitEvent` 使用 Bevy Observer/Trigger：

```text
commands.trigger(AbilityTaskEvent::new(...))
```

外部可以通过 `app.add_observer(...)` 监听 `AbilityTaskEvent`，并在 observer 中生成 projectile、area zone 或其它玩法实体。

这里没有使用 message 队列，因此触发语义更接近即时 observer。需要注意的是，observer 内仍然应遵守 Bevy ECS 的命令应用时机。

## 队列系统

项目有两个运行期队列。

### AbilityActivationQueue

保存：

- source
- target
- ability handle

每个 fixed tick 最多处理 `ABILITY_ACTIVATION_QUEUE_MAX_PER_TICK` 个请求。默认值在 `GameplayAbilitySystemSettings` 中，为 64。

队列为空时，`ability_activation_queue_has_work` 会让消费系统跳过本 tick。

### GameplayEffectApplicationQueue

保存：

- target
- effect
- payload

每个 fixed tick 最多处理 `GAMEPLAY_EFFECT_APPLICATION_QUEUE_MAX_PER_TICK` 个请求。默认值为 64。

AbilityTask 中的 `ApplyGameplayEffect` 不会直接应用 effect，而是把请求推入 `GameplayEffectApplicationQueue`。这样 effect 应用有统一入口，也方便做限流、排序或调试。

队列为空时，`gameplay_effect_application_queue_has_work` 会让消费系统跳过本 tick。

## 典型链路：技能造成直接伤害

```text
角色实体拥有 AbilitySystemComponent、AttributeSet、GameplayTagContainer
  -> ASC 被授予一个 GameplayAbility
  -> 外部调用 try_activate_ability_by_handle 或推入 AbilityActivationQueue
  -> can_activate_ability 检查 tag、cooldown、cost
  -> commit_ability 执行 cost 和 cooldown
  -> activation effects 或 AbilityTask 应用伤害 GameplayEffect
  -> GameplayEffect 生成 spec
  -> Instant modifier 修改目标 AttributeSet base
  -> 目标属性读取时懒重算，得到最新 current
```

## 典型链路：子弹技能

推荐做法是让角色 ASC 激活技能，让子弹作为普通玩法实体携带命中逻辑和 payload，不给每个子弹挂 ASC。

```text
角色 ASC 激活 Ability
  -> Ability commit cost/cooldown
  -> startup task 或 AbilityTaskEvent observer 生成子弹实体
  -> 子弹保存 EffectPayload，或保存足够信息用于命中时创建 payload
  -> 子弹碰撞敌人
  -> 调用 apply_gameplay_effect，或推入 GameplayEffectApplicationQueue
  -> GameplayEffect 使用 payload/source_snapshot 计算伤害
  -> 敌人 AttributeSet 应用 instant modifier
```

如果子弹伤害需要以发射瞬间为准，发射时创建 `AttributeSetSnapshot` 并放入 payload。

如果子弹伤害需要以命中瞬间为准，命中时再通过 `EffectContext` 查询 source 当前属性。

## 典型链路：分裂弹头和持续范围伤害

推荐链路：

```text
主弹发射时生成 EffectPayload
  -> payload 可携带 source_snapshot，冻结发射时攻击力和元素加成
  -> 主弹命中敌人
  -> 对命中敌人应用主伤害 GameplayEffect
  -> 生成 3 个分裂子弹
  -> 分裂子弹复用或派生 payload
  -> 分裂子弹命中后生成 AreaDamageZone
  -> AreaDamageZone 每 0.5s 对半径内敌人推送 GameplayEffectApplicationQueue
  -> 持续 3s 后销毁
```

这个模型里：

- Ability 只负责发射行为和资源消耗
- Projectile 负责飞行和碰撞
- AreaDamageZone 负责范围查询和周期触发
- GameplayEffect 负责最终伤害计算和属性修改
- EffectPayload 负责把 source、causer、level、snapshot 等语义传到最终伤害点

## 当前设计约定

- Attribute 读取采用懒重算。
- `recalculate_attribute_sets_system` 仍保留在 FixedUpdate 末尾，但只处理 `Changed<AttributeSet>`。
- Instant modifier 修改 base。
- Duration modifier 进入 Aggregator，并由 ActiveEffectHandle 管理生命周期。
- Periodic effect 每次 tick 执行 instant modifier。
- Effect 数值在 `make_spec` 时冻结。
- Payload 是跨系统传递的数据，Context 是 spec 计算时的临时视图。
- Ability 是定义，AbilityTask 是执行层。
- AbilityTask 通过 Trigger/Observer 发事件，不走 message。
- ActiveAbility 结束时会清理子 AbilityTask。
- ActiveEffect 作为 target 子实体存在，并额外维护 target 索引；modifier/tag 依赖显式 cleanup，index 会在 cleanup 和 removed-components 兜底系统中同步。
- 子弹、范围区、陷阱这类实体通常不需要 ASC，除非它们本身确实要拥有技能、属性和完整状态生命周期。

## 已完成的运行时优化

当前版本在不添加新玩法功能的前提下完成了这些运行时优化：

- `FixedUpdate` 从整条 `.chain()` 改为 `GameplayAbilitySystemSet`，减少不必要的串行调度。
- Ability 和 Effect 队列增加 `run_if`，空队列时不进入消费系统。
- `ActiveGameplayEffectTargetIndex` 记录 target 到 active effect handles 的映射，减少按 target 查询时的全局扫描。
- Active effect target 索引增加 handle 反查和 removed-components 兜底同步。
- `AbilitySystemComponent` 维护 ability handle 索引，按 handle 查找不再线性扫描。
- `GameplayTagContainer` 支持直接 bitset 匹配，减少临时容器构造。
- application immunity、ability/effect tag 匹配路径避免临时创建完整 `GameplayTagContainer`。
- `AttributeSet` 使用 `Vec<Option<Attribute>>` 存储属性，减少每个属性独立装箱。
- `recalculate_attribute_sets_system` 只处理 `Changed<AttributeSet>`。
- Active effect cleanup 只移除该 effect 实际涉及的 attribute modifier。
- Ability 激活主路径复用已准备的 cost plan，避免 cost spec 重复构建。

## 仍可继续优化的方向

当前系统主流程已经比较完整，但还有一些值得继续演进的点：

- `AbilityTaskDef` 类型还较少，可以扩展 `WaitEvent`、`WaitCollision`、`SpawnProjectile`、`SpawnArea` 等任务。
- `EffectPayload` 的 snapshot 创建时机需要由上层玩法明确约定，例如发射时、命中时、区域创建时。
- `StackExpirationPolicy::RemoveSingleStack` 当前使用共享 duration，不是每层独立 duration；如果需要更完整的堆叠到期规则，需要引入 per-stack 时间记录。
- `ActiveGameplayEffect` 依赖显式 cleanup 移除 modifier/tag；外部直接 despawn effect entity 会绕过清理逻辑。
- cost 检查目前假设 cost effect 全部是 Add modifier，复杂资源消耗可以独立扩展。
- Effect、Ability、Task 目前主要是代码构造，未来可以继续做 asset/serde/data-driven 支持。

## 快速检查

常用检查命令：

```bash
cargo fmt
cargo clippy
cargo test
cargo build
```
