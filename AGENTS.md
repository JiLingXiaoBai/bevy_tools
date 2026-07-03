# AGENTS.md — bevy_tools

## 项目概述

`bevy_tools` 是一个为 [Bevy](https://bevyengine.org/) 游戏引擎打造的
**Gameplay Ability System (GAS)** 库，设计灵感来源于虚幻引擎的 GAS 框架。
它提供了模块化的 ECS 友好架构，用于构建复杂的 RPG/MOBA/ARPG 游戏机制。

## 技术栈

- **语言：** Rust (edition 2024)
- **引擎：** Bevy 0.19
- **额外依赖：** `rand` 0.10.1
- **许可证：** MIT

## 项目目标：

- 高性能
- ECS 优先架构
- 易于维护
- 数据驱动、尽可能保持确定性（Deterministic）
- 尽量减少不必要的第三方依赖

## 开发原则

所有设计遵循以下优先级：

1. 正确性（Correctness）
2. 可读性（Readability）
3. 性能（Performance）

不要为了微小的性能收益而牺牲代码的正确性和可维护性。

当存在多种实现方案时，优先选择更简单、更容易理解的方案；除非经过性能分析（Profiling）证明存在瓶颈，否则不要进行过早优化。

## 编码规范

### 通用规范

- 遵循 `rustfmt` 格式化规范。
- 函数应保持职责单一、长度适中。
- 避免过深的嵌套逻辑。
- 命名应具有明确语义，避免无意义缩写。
- 所有公开 API（Public API）应编写文档注释。

### 错误处理

- 优先使用 `Result` 返回错误。
- 除非明确确认不会失败，否则不要使用 `unwrap()` 或 `expect()`。
- Library 代码中禁止主动 `panic!()`。

### 所有权与内存

- 优先使用 Borrow，而不是 Clone。
- 避免不必要的内存分配。
- 避免无意义的堆内存（Heap）分配。

## 依赖管理

优先使用 Rust 标准库以及 Bevy 官方提供的功能。

除非确实能够带来明显收益，否则不要新增第三方依赖。

如果必须新增依赖，请说明新增原因以及带来的价值。

## Unsafe

除非明确要求，否则不要使用 `unsafe`。

始终优先选择 Safe Rust。

## ECS 设计规范

始终以 ECS 思维进行设计。

优先使用：

- Component
- Resource
- System
- Event / Message

避免使用不符合 ECS 思想的面向对象设计。

各个 System 应尽可能保持独立，减少耦合。

## 性能规范

性能是项目的重要目标。

优先考虑：

- 栈内存（Stack）
- 合适情况下使用固定大小数组
- Cache Friendly（缓存友好）的数据布局
- 连续且可预测的内存访问

避免：

- 过度 Clone
- 不必要的动态分发（Dynamic Dispatch）
- 热路径中的堆内存分配

所有性能优化应建立在实际测试和 Profiling 的基础上。

## 确定性（Determinism）

Gameplay 逻辑应尽可能保持确定性。

避免依赖：

- HashMap 等容器的遍历顺序
- 不同平台可能存在差异的浮点行为
- 隐藏的全局状态

## 序列化

如需序列化功能，优先使用：

- serde
- ron

配置应采用数据驱动方式管理。

除非确有必要，否则不要序列化 Trait Object。

## 日志

如需日志，统一使用 `tracing`。

运行时代码中不要使用 `println!()` 输出日志。

## 测试

完成开发后，应至少执行以下检查：

```bash
cargo fmt
cargo clippy
cargo test
cargo build
```

尽量保证代码无编译警告，并通过所有测试。

## 文档规范

所有公开 API 应说明：

- 功能
- 参数
- 返回值

复杂算法应补充设计思路或实现说明，便于后续维护。

## AI 协作规范

当 AI 修改代码时，应遵循以下原则：

- 保持现有项目架构不变。
- 保持已有代码风格一致。
- 不进行无关的重构。
- 尽量缩小修改范围。
- 对重要设计决策进行说明。
- 未经要求，不主动修改公共 API 名称。

如果需求存在歧义，应先询问，而不是擅自进行架构调整。

## 提交原则

优先进行小而明确的修改。

每次提交（Commit）只解决一个独立问题。

避免将功能开发与代码重构混在同一次修改中。

## 项目价值观

本项目始终坚持以下原则：

- 正确性（Correctness）
- 可维护性（Maintainability）
- ECS 优先
- Gameplay 确定性（Determinism）
- 高性能（Performance）
- 编写符合 Rust 风格的代码
- 尽量减少第三方依赖

## 架构

```
src/
├── lib.rs                      # 插件定义、公共重导出
├── main.rs                     # 示例用法 / 冒烟测试
├── gas/                        # 游戏性技能系统 (核心)
│   ├── gameplay_tags/          # 层级标签系统 (基于位集)
│   ├── attributes/             # 属性系统 (含修饰器聚合)
│   ├── modifiers/              # 修饰器操作与幅度
│   ├── gameplay_effects/       # 游戏效果 (即时/持续/无限)
│   ├── gameplay_abilities/     # 技能 (冷却、消耗、激活效果、任务)
│   ├── ability_system/         # AbilitySystemComponent (ASC) + 队列
│   └── settings.rs             # 全局常量
├── randoms/                    # 确定性随机数封装 (Bevy Resource)
├── unique_names/               # 字符串驻留池 (hash → u32)
└── tests/                      # 测试代码（按被测模块分文件）
```

### Bevy 插件 (位于 `lib.rs`)

| 插件                                 | 类型        | 功能                                              |
| ------------------------------------ | ----------- | ------------------------------------------------- |
| `UniqueNamePlugin`                   | Plugin      | 提供 `UniqueNamePool` 资源                        |
| `GameplayTagPlugin`                  | Plugin      | 提供 `GameplayTagManager` 资源                    |
| `RandomPlugin`                       | Plugin      | 提供 `Random` 资源                                |
| `GameplayAbilitySystemRuntimePlugin` | Plugin      | 核心运行时：初始化所有资源并注册 FixedUpdate 系统 |
| `GameplayAbilitySystemPlugin`        | PluginGroup | 组合以上四个插件                                  |

`GameplayAbilitySystemRuntimePlugin` 通过 `SystemSet` 组织 FixedUpdate 系统，
按以下顺序执行：

| SystemSet                     | 系统                                               | 职责                                     |
| ----------------------------- | -------------------------------------------------- | ---------------------------------------- |
| `UpdateEffectTagRequirements` | `update_active_effect_tag_requirements_system`     | 效果抑制/移除的标签条件检查              |
| `EffectTicks`                 | `tick_effect_duration_system`                      | 倒计时并过期持续效果                     |
|                               | `tick_effect_period_system`                        | 周期性执行修饰器                         |
| `AbilityTasks`                | `tick_ability_tasks_system`                        | 推进技能任务 (等待/立即)                 |
| `Queues`                      | `process_gameplay_effect_application_queue_system` | 消耗效果应用队列 (有工作时)              |
|                               | `process_ability_activation_queue_system`          | 消耗技能激活队列 (有工作时)              |
| `Cleanup`                     | `cleanup_finished_abilities_system`                | 清理 Ending/Cancelled 状态的技能         |
|                               | `reconcile_active_effect_target_index_system`      | 从索引中清除已移除的效果                 |
| `RecalculateAttributes`       | `recalculate_attribute_sets_system`                | 重算所有脏属性 (Changed\<AttributeSet\>) |

`RuntimePlugin` 还初始化以下 Resource：

- `AttributeIdManager`
- `AbilityActivationQueue` (每 tick 上限 64，可配置)
- `GameplayEffectApplicationQueue` (每 tick 上限 64，可配置)
- `ActiveGameplayEffectTargetIndex`

### 游戏性标签 (`gas/gameplay_tags/`)

层级标签以 **位集 (bitset)** 存储，实现 O(1) 查询。父标签自动传播到子标签
（例如 `Effect.Debuff.Stun` 蕴含 `Effect.Debuff` 和 `Effect`）。

- `GameplayTag` — 封装 `u16` 位索引的轻量包装
- `GameplayTagContainer` — 每实体的 Component，含位集 + 引用计数；
  提供 `has_tag`、`has_all`、`has_any` 及对应的 `*_bits` 变体
- `GameplayTagManager` — 全局 Resource；注册标签并追踪继承关系、
  返回 `GameplayTagError` 错误
- `GameplayTagRegister` — `SystemParam`，通过点分名称注册标签
  （如 `"Effect.Debuff.Stun"`），返回 `Result<GameplayTag, GameplayTagError>`，
  递归自动注册父标签
- 最大标签数：`GAMEPLAY_TAG_SIZE = 512`（可在 `settings.rs` 中配置）

### 属性 (`gas/attributes/`)

- `AttributeId` — `u16` 句柄（通过 `AttributeIdManager` / `AttributeIdRegister` 注册）
- `Attribute` — `base` 值 + `evaluated`（聚合后）+ `current`（clamp 后）+
  `Aggregator`（修饰器栈）+ `dirty` 标记 + `AttributeClamp`；
  `get_current_value(&mut self)` 自动调用 `recalculate()`
- `AttributeSet` — 固定大小 Vec Component (`ATTRIBUTE_SET_SIZE = 256`)；
  拥有自己的 `dirty` 标记，支持 `Changed<AttributeSet>` 变更检测；
  提供 `remove_modifiers_for_attributes()` 按属性 ID 精确移除
- `Aggregator` — 按规范顺序计算修饰器：**Override → Add → PercentAdd → Multiply**；
  支持自定义 `executor` 函数指针
- `AttributeClamp` — `None` 或 `Range { min: Option<f64>, max: Option<f64> }`
  （纯静态值，不再有 AttributeClampBound 引用其他属性）
- `AttributeSetSnapshot` — 属性快照组件，记录快照时刻的 base/current 值
- `AttributePostExecute` — 回调类型 `fn(&mut AttributeSet, AttributeId, f64, f64)`

### 修饰器 (`gas/modifiers/`)

| 操作         | 说明               |
| ------------ | ------------------ |
| `Add`        | 直接加到基础值     |
| `PercentAdd` | 当前值的百分比加成 |
| `Multiply`   | 乘法缩放           |
| `Override`   | 直接覆盖该值       |

- `Modifier` — 修饰器定义（属性ID + 操作 + 幅度）
- `ModifierMagnitude` — 幅度可以是 `Flat(f64)` 或 `Calculated(Box<dyn Trait>)`，
  通过 `EffectContext` 解析为具体值
- `ModifierSpec` — 已解析的不可变修饰器规格；`.scaled_by_stack()` 返回堆叠后的副本
- `AppliedModifier` — 已应用到聚合器的修饰器，关联 `ActiveEffectHandle`

### 游戏效果 (`gas/gameplay_effects/`)

核心的 Buff/Debuff 系统。三种持续时间类型：

- **Instant** — 一次性应用修饰器（直接修改 base 值）
- **DurationTicks(u32)** — 持续 N 个 fixed-update tick 后自动移除
- **Infinite** — 持久存在，直到显式移除

关键特性：

- **堆叠 (StackingPolicy)** — `StackingType::None` / `AggregateBySource` / `AggregateByTarget`，
  包含独立可配的：幅度策略 (None/Linear)、持续时间策略 (KeepExisting/RefreshOnSuccessfulStack)、
  周期策略 (KeepCurrentTick/ResetOnSuccessfulStack)、溢出策略 (RejectApplication/RefreshDuration)、
  过期策略 (RemoveAllStacks/RemoveSingleStack)
- **周期性执行** — `EffectPeriodTicks` 定义每 N tick 应用一次修饰器；
  可选是否在应用时立即执行 (`execute_on_applied`)
- **标签要求** — `TagRequirements` 含 `require_all` 和 `ignore_any`，
  预缓存位集(`require_all_bits`/`ignore_any_bits`)加速查询；
  应用于来源/目标的应用条件、持续条件、移除条件
- **应用免疫** — 激活的效果可对匹配来源标签 + 效果标签的传入效果授予免疫
- **移除标签** — `remove_effects_with_tags` 指定标签触发先移除再应用
- **抑制 (Inhibition)** — 效果在持续条件不满足时暂时移除修饰器和授予的标签，
  条件恢复后自动恢复

#### API 流程

`apply_gameplay_effect(target, effect_def, params, &payload)` 便捷函数，
内部调用 `prepare_gameplay_effect()` → `execute_gameplay_effect_plan()`。

`prepare_gameplay_effect()` 依次检查：概率 → 应用条件 → 免疫 → 生成 Spec →
查找可堆叠效果 → 收集待移除效果 → 返回 `GameplayEffectApplicationPlan`

`GameplayEffectApplicationKind` 枚举三种执行路径：
`Instant` / `StackExisting { handle, new_stack_count }` / `CreateActive`

#### 效果应用队列

`GameplayEffectApplicationQueue` (Resource) 支持延迟批量应用，
`process_gameplay_effect_application_queue_system` 每 tick 消费最多 64 个请求。

### 游戏技能 (`gas/gameplay_abilities/`)

- `GameplayAbility` — 技能定义：`AbilityTags`、`startup_tasks: Vec<AbilityTaskDef>`、
  可选冷却效果、可选消耗效果、激活效果列表、`end_on_activation`、
  `allow_multiple_instances`
- `AbilityTags` — 资源标签 (`ability_asset_tags`)、取消标签 (`cancel_abilities_with_tags`)、
  阻止标签 (`block_abilities_with_tags`)、激活要求标签 (`activation_required_tags`)、
  激活阻止标签 (`activation_blocked_tags`)
- `GameplayAbilitySpec` — 运行时实例：句柄 (`AbilitySpecHandle`)、`Arc<ability>`、
  等级、输入绑定 (`input_id: Option<u16>`)、活跃计数
- `ActiveGameplayAbility` — 技能激活时生成的 Component；
  追踪 `source`、`spec_handle`、`target`、`status`

#### 激活流程

`try_activate_ability_by_handle()` →

1. 取消匹配 `cancel_abilities_with_tags` 的活跃技能
2. 检查活跃计数（多实例控制）
3. `passes_ability_activation_requirements()` — 阻止标签、激活阻止标签、
   激活要求标签、冷却标签检查
4. `prepare_ability_commit_plans()` — 准备消耗和冷却计划
5. `start_ability()` — 设置阻止标签、递增活跃计数、生成 `ActiveGameplayAbility` 实体
6. `execute_ability_commit_plans()` — 执行消耗 + 冷却效果；失败则回滚
7. 应用 `activation_effects` (best-effort)
8. `spawn_startup_ability_tasks()` — 生成启动任务
9. 如 `end_on_activation`，将状态设为 `Ending`

状态生命周期：`Activating` → `Active` → `Ending`/`Cancelled` → 清理销毁。

#### 技能激活队列

`AbilityActivationQueue` (Resource) 支持延迟批量激活，
`process_ability_activation_queue_system` 每 tick 消费最多 64 个请求。

### 技能任务 (`gas/gameplay_abilities/ability_task.rs`)

技能可附带 `AbilityTaskDef` 列表来编排时间线行为：

- `Instant` — 立即完成，触发 `on_finished` 回调
- `WaitTicks { ticks }` — 等待指定 tick 后完成

`on_finished` 可以是：

- `EndAbility` — 结束技能
- `EmitEvent { event_id }` — 触发 `AbilityTaskEvent` (Bevy Event)
- `ApplyGameplayEffectToTarget { effect }` — 通过队列应用效果
- `ActivateAbility { handle }` — 通过队列激活另一个技能

`tick_ability_tasks_system` 在 `AbilityTasks` set 中运行，推进所有任务的 tick。

### 能力系统组件 (`gas/ability_system/`)

`AbilitySystemComponent` (ASC) 是挂载到可使用技能的实体上的主要 Component：

- `abilities: Vec<GameplayAbilitySpec>` — 已授予技能
- `ability_indices: HashMap<AbilitySpecHandle, usize>` — O(1) 查找索引
- `blocked_ability_tags: GameplayTagContainer` — 激活阻止标签的技能时设置的标签

公开方法：`give_ability()`、`clear_ability()`、`find_ability_spec()` 等。

`AbilitySystemParams` 是主要的 `SystemParam` 聚合器——打包了所有 GAS
操作所需的 `Commands`、查询和资源（包括 `active_effect_target_index`、`time` 等）。

### 效果上下文 (`EffectContext` 与 `EffectPayload`)

`EffectPayload` 携带效果执行所需信息：`source: Entity`、`causer: Option<Entity>`、
`level: u32`、`source_snapshot: Option<AttributeSetSnapshot>`。

`EffectContext` 包装 `EffectPayload` 加上世界查询引用（只读），
为 `ModifierMagnitudeCalculation` 等动态计算提供数据访问。

### 唯一名称 (`unique_names/`)

使用 `FixedHasher` 的字符串驻留池（hash → u32 索引）。
`UniqueNamePool` 是 Bevy `Resource`。在 debug 构建中，哈希冲突会触发 panic。
空字符串预留在索引 0。

### 随机数 (`randoms/`)

`Random` 是 Bevy `Resource`，封装 `StdRng`，默认固定种子 (`123456`)。
提供 `random_range`、`random_bool`、`from_rng`、
`sample_interior`/`sample_boundary` 用于形状采样。

## 测试组织

所有测试代码放在项目根目录的 `tests/` 目录下，按被测模块拆分为子目录和独立文件：

```
tests/
├── gas_tests/                      # GAS 核心模块测试
│   ├── gameplay_tags_test.rs       # 测试 gameplay_tags 模块
│   ├── attributes_test.rs          # 测试 attributes 模块
│   ├── modifiers_test.rs           # 测试 modifiers 模块
│   ├── gameplay_effects_test.rs    # 测试 gameplay_effects 模块
│   ├── gameplay_abilities_test.rs  # 测试 gameplay_abilities 模块
│   └── ability_system_test.rs      # 测试 ability_system 模块
├── randoms_tests/                  # 测试 randoms 模块
└── unique_names_tests/             # 测试 unique_names 模块
```

每个测试文件使用 `#[cfg(test)] mod tests { ... }`
组织结构，但也可以根据需要直接写在文件顶层。

## 编码约定

- **Rust edition 2024** — 使用新语言特性（如 `if let` 链、`use` 重导出、
  `impl Trait` 在关联类型位置等）
- **`pub use` 重导出模式** — 每个模块有一个 `mod.rs` 声明子模块并通过
  `pub use submodule::*` 重导出其公开项
- **Component/Resource 为中心** — 游戏状态存储在 Bevy Component 和 Resource
  中，而非独立的 world 存储
- **SystemParam 作为公共 API** — 函数如 `apply_gameplay_effect()` 接收
  `&mut AbilitySystemParams` 而非单独的查询
- **Arc\<GameplayEffect\>/Arc\<GameplayAbility\>** — 效果和技能定义通过 `Arc`
  共享；规格通过 `Arc::ptr_eq` 比较
- **脏标记模式** — `Attribute` 和 `AttributeSet` 都有 `dirty: bool`，
  `recalculate_attribute_sets_system` 使用 `Changed<AttributeSet>` 过滤
- **tick 计时，非秒** — 所有时间（持续时间、周期、任务等待）以 `FixedUpdate` tick
  为单位
- **标签引用计数** — `GameplayTagContainer` 追踪每个位被设置的次数，
  避免重叠的效果授予/移除互相干扰
- **`debug_assert!`** 用于内部不变量；超出容量时返回 `Err` 而非直接 panic
  （`GameplayTagError`、`AttributeIdError`）
- **队列模式** — 技能激活和效果应用支持通过队列延迟到下一 tick 批量执行，
  避免在同一帧内递归执行导致的借用问题

## 新增功能指南

1. **新的修饰器操作** — 在 `ModifierOperation` 中添加变体，
   在 `Aggregator::apply_modifier_spec` 和 `default_executor` 中处理
2. **新的堆叠策略** — 在相应的 `Stack*` 枚举中添加变体，
   在 `find_stackable_active_effect` 和 `execute_stack_existing_effect` 中处理
3. **新的属性** — 通过 `AttributeIdRegister::request_or_register_attribute_id` 注册，
   在 `AttributeSet::initialize_attribute` 中初始化，通过 `AttributeId` 引用
4. **新的游戏性标签** — 通过 `GameplayTagRegister::request_or_register_tag` 注册
5. **新的系统** — 添加到 `GameplayAbilitySystemRuntimePlugin::build()` 中，
   选择合适的 `GameplayAbilitySystemSet` 并通过 `.in_set()` 或 `.before()`/`.after()`
   满足顺序要求
6. **新的 AbilityTask 类型** — 在 `AbilityTaskDef` 和 `AbilityTaskKind` 中添加变体，
   在 `tick_ability_tasks_system` 和 `AbilityTaskOnFinishedDef::instantiate` 中处理
