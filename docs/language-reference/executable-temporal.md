# 可执行 Temporal

`.veac` 是动画的唯一 authoring source of truth。动态叶子既可由 root 的 absolute `animate` 声明
产生，也可由 module、function、method 内 owner-relative 的 typed attachment 产生。不从 JSON IR、
host property bag 或 render 参数注入，也不会在 render 时重新解释 Surface AST。完整路径是：

```text
animate declaration or attachment expression
  -> typed sink + stable logical owner
  -> compile_temporal_expression
  -> verified Core v10
  -> bounded residualization
  -> canonical TemporalProgram + TemporalBinding
  -> Animatable::Binding on the approved sink
  -> complete ProjectEnvelope validation
  -> planner -> backend
```

整个 build 原子执行。任一表达式、owner、sink、预算或 canonical validation 失败时，不会发布部分
Project、Temporal library、cache entry 或 source-edit preview。

## 声明语法

目标使用完整稳定逻辑路径。属性与 target kind 都来自闭合集合，不接受 JSON pointer 或字符串字段：

```veac
fn pulse(value: scalar) -> scalar {
  clamp(value * 2.0, 0.0, 1.0)
}

animate visual-opacity on clip(@demo, @main, @visual, @hero) {
  pulse(progress)
}
```

root 声明使用完整 logical path。组件 factory 则直接使用刚构造的 typed owner handle：

```veac
fn animated_card(key: identifier, at: time) -> Item {
  let card = item(key, item_enabled(), during(at, 1s),
    source_generated(generator_solid(#c43a69ff)), source_timing_native());
  animate visual-opacity on clip(card) {
    clamp(progress * 2.0, 0.0, 1.0)
  }
}
```

attachment expression 返回原 owner handle，因此可直接作为 factory 结果或继续 method chain。它只能
绑定当前 graph 中尚未被 owner 消费的 `Item`/`Apply`；freeze 后才把 handle 解析为 absolute canonical
sink。同一 factory 的两个实例共享 verified closure definition 及内容相同的 canonical program，
但得到各自不同且稳定的 binding、provenance 与 logical key。module-private 或 imported `Pure` helper
会进入 closure 的 transitive content identity；不生成源码、不重解析
`.veac`，也没有隐藏 `animate_*` callee、字符串 relative path 或 property bag。

同一目标属性只能有一个 producer，包括 root 声明与组件 attachment 之间的冲突。动画 body 是普通
typed expression block，可以调用已解析的 `Pure` 函数、method 和闭合构造器，也可以使用比较和惰性
分支；`LocalMutation`、`GraphEmit` 与 Temporal-stage topology 均在 verified Core 前拒绝。

需要 source clock 时必须显式绑定目标 project 内的 resource：

```veac
animate visual-opacity on clip(@demo, @main, @visual, @hero-clip)
  using resource(@demo, @hero) {
  clamp(source_time / 2s, 0.0, 1.0)
}
```

resource 必须正是目标 clip 的 canonical media owner。generated source、其他 resource 或其他
project 都 fail closed。

`clip`、`text`、`clip-mask`、`clip-effect` 是 Item owner；`apply`、`apply-mask`、`apply-effect`
是 Sequence owner。完整 target 参数、属性矩阵和 optional-leaf 规则见
[Temporal 动态叶矩阵](executable-temporal-sinks.md)。

## 隐式输入

每个声明可见以下 typed symbol：

| symbol | 类型 | canonical owner |
| --- | --- | --- |
| `sequence_time` | `time` | `SequenceId` |
| `clip_time` | `time` | `ItemId` |
| `frame` | `integer` | `SequenceId` |
| `progress` | `scalar` | `ItemId` |
| `source_time` | `time` | 目标 `ItemId` |

root 声明中的 `source_time` 只在 `using resource(...)` 时存在。Item attachment 的闭包签名固定包含
五个 typed 参数，但 residualizer 只为 body 实际引用的参数发布 input；因此 generated/text Item 不引用
`source_time` 时不会伪造 source clock，真正引用时则必须从 canonical Item source 推导出 media owner。
Apply attachment 只公开 `sequence_time: time` 与 `frame: integer`。canonical Temporal input 只包含
这些 clock 与 typed Parameter。`.veac` 的 `input analysis name: T` 是 Build-stage typed input，在
residualization 前已经成为 concrete value，并不是随时间采样的 analysis signal。schema v10 不发布
缺少 analyzer、采样时钟、插值和数据传输语义的 Analysis placeholder；旧 `analysis` variant 或
`analyses` binding field 会被 deny-unknown validation 拒绝。

五种 clock 都经过相同的 authored binding 合同：`sequence_time`、`frame` 绑定
`TemporalClockOwner::Sequence`；`clip_time`、`progress`、`source_time` 绑定
`TemporalClockOwner::Item`。类型、owner、program input 与 canonical binding 必须成对一致，
reference evaluator 和 backend 使用同一映射。

## 曲线采样

曲线是闭合 typed builtin，不是 property bag，也不接受字符串 easing 名称。所有 key 必须在 Build
阶段已知，位置与采样输入同类型，且严格递增：

```veac
animate visual-opacity on clip(@demo, @main, @visual, @hero) {
  sample_curve_ease_in_out(progress, [
    (0.0, 0.0),
    (0.35, 1.0),
    (1.0, 0.0),
  ])
}
```

公开函数形成闭合集合：

```text
sample_curve_hold(input, keys)
sample_curve_linear(input, keys)
sample_curve_ease_in(input, keys)
sample_curve_ease_out(input, keys)
sample_curve_ease_in_out(input, keys)
sample_curve_spring(input, keys, frequency, decay, initial_velocity)
sample_curve_cubic_bezier(input, keys, x1, y1, x2, y2)
```

`input` 只能是 `scalar` 或 `time`；`keys` 的类型是
`list<(scalar, T)>` 或 `list<(time, T)>`。`T` 支持 `scalar`、`percent`、`length`、`angle`、
`color`、`Vector`、`Point`、`Rect`。同一曲线的值类型和 length unit 必须一致。key 列表不能为空，
不能超过 canonical IR 上限；spring 与 cubic-bezier 参数必须是有限、合法的 Build-stage `scalar`。

这些函数只能在 Temporal residualization 中使用。普通 Build/Const 求值没有 clock，调用会返回
`EXPRESSION_TEMPORAL_CURVE_CONTEXT`。编译路径是 verified Core builtin 到
`TemporalNodeKind::CurveSample`；JSON 只保存 canonical IR，不是曲线的 authoring source。

## 闭合属性

目标类型由属性和 target kind 共同决定，源码不能自行声明或覆盖。完整合同见
[Temporal 动态叶矩阵](executable-temporal-sinks.md)。`Vector` 在 canonical Temporal ABI 中闭合映射为
`TemporalType::Vec2`。所有 owner、mask ordinal、effect/stage key、parameter kind 与 optional leaf 都必须
由 Build graph 已经创建；Temporal dependency 只能替换现有 `Animatable<T>` 值。由此保持
`static topology, dynamic leaf values`。

## Residualization

普通 build runtime 不把 Temporal input 冒充 concrete value。编译器对动画 body 的 verified Core 做
partial evaluation：已知 Const/Build 值先求值，Temporal dependency 变成 typed DAG input；算术、比较、
惰性分支、`min`/`max`/`clamp`、typed curve 和闭合 vector/point/rect compose/project 生成 canonical node。

用户函数按 numeric `FunctionId` 从 verified registry 内联 residualize。只有 `Pure` 调用允许进入程序；
`GraphEmit`、`LocalMutation`、动态 topology、Temporal identifier、动态 clamp 边界、未支持 operation、
非闭合值和超预算都在发布前拒绝。最终结果必须仍依赖至少一个 Temporal input，并与 sink 类型完全一致。

内容相同的 residual program 按 canonical digest 池化。program ID、binding ID、provenance ID、definition
ID、source ID 与 logical key 都使用 domain-separated stable identity；同一 ID 指向不同内容时失败。
`declared_inputs_sha256` 和 `source_graph_sha256` 覆盖 authored declaration，因此编辑 body 会使 cache
identity 失效并完整 rebuild。

## Source Edit

`source-index` 把动画 body 发布在完整 typed temporal declaration target 的 `temporal_animation` site 上，
并携带闭合 property；mask
ordinal、apply/stage/effect/parameter identity 不会退化为 byte offset。`source-edit` 以 revision-bound
`set_body` 修改 `.veac`，随后重新解析、验证 Core、执行静态
graph、residualize 并验证 envelope；dry-run 不写文件，失败不会产生部分修改。

`ExecutableTemporalLeaf` 只保留为编译器内部 typed carrier，负责把 authored `TemporalDecl` 连接到
lowering。它不是用户 authoring API。canonical JSON 只承载已验证的 program/binding/provenance ABI；
backend 只消费 `ProjectEnvelope::canonical_json` 的结果，不定义或反编译 VEAC 语义。
