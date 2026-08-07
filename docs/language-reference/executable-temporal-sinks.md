# Temporal 动态叶矩阵

`animate` 只绑定 canonical IR 已公开且 backend 可消费的 `Animatable<T>` 叶子。root target 使用
absolute logical path；组件 target 使用 owner handle 和 typed local selector。两者来自同一闭合
primitive catalog，不是 property bag：

| target | 参数 | 允许属性 |
| --- | --- | --- |
| `clip` | project, sequence, layer, item | `visual-*`、`audio-*` |
| `text` | project, sequence, layer, item | `text-*` |
| `clip-mask` | item path, mask ordinal | `mask-*` |
| `clip-effect` | item path, effect key, closed effect parameter | `effect-parameter` |
| `apply` | project, sequence, apply key | `apply-opacity` |
| `apply-mask` | apply path, mask ordinal | `mask-*` |
| `apply-effect` | apply path, stage key, effect key, closed effect parameter | `effect-parameter` |

组件形式分别是 `clip(item)`、`text(item)`、`clip-mask(item, ordinal)`、
`clip-effect(item, effect, parameter)`、`apply(apply)`、`apply-mask(apply, ordinal)` 与
`apply-effect(apply, stage, effect, parameter)`。ordinal 是 `integer`，其余 selector 是
`identifier`；owner 类型、arity 与 selector 类型写入 Core v10 并由 verifier 重验。

示例：

```veac
animate mask-scale on clip-mask(@demo, @main, @visual, @hero, 0) {
  vector(0.5 + progress * 0.5, 1.0)
}
animate effect-parameter on clip-effect(
  @demo, @main, @visual, @hero, @blur, @radius
) { progress * 12.0 }
animate apply-opacity on apply(@demo, @main, @global-grade) {
  clamp(sequence_time / 2s, 0.0, 1.0)
}
```

属性类型是闭合合同：

| 属性族 | 属性 | 结果类型 |
| --- | --- | --- |
| visual | position / scale / rotation / crop / opacity | `Point` / `Vector` / `Angle` / `Rect` / `Scalar` |
| audio | gain / pan | `Scalar` |
| mask | position / scale / rotation / feather / expansion | `Vector` / `Vector` / `Angle` / `Scalar` / `Scalar` |
| text | position / scale / rotation / reveal / highlight-progress / opacity | `Point` / `Vector` / `Angle` / `Scalar` / `Scalar` / `Scalar` |
| effect | parameter | `Scalar`，且所选 `EffectParameter` 必须对应已有的 `Animatable<f64>` 叶子 |
| apply | opacity | `Scalar` |

mask ordinal 是 Build-stage list topology；effect 与 stage 是受 name contract 约束的 local ID。parameter
identifier 必须在 DSL 边界解析为闭合 `EffectParameter`；编译器从同一 logical path 推导 canonical ID，
再核对实际 owner，不按 JSON field name 反射，也不保留字符串 property key。

以下情况全部 fail closed：owner 或 ordinal 不存在；text animation、highlight、crop 等 optional leaf 缺失；
effect parameter 不属于闭合集，或不是所选 effect 已有的 `Animatable<f64>` 叶子；property 与 target kind
不匹配；同一完整 sink 有两个 producer。
声明不能创建 mask、effect、text animation、apply 或 parameter variant。

Item target 可使用 sequence/frame/clip/progress clock，并可显式绑定 source clock。Apply target 没有 Item
owner，只公开 `sequence_time` 与 `frame`；在 apply body 中引用 clip/progress/source symbol 会在表达式编译期
失败。planner 与 FFmpeg backend 对上述 sink 使用 canonical `TemporalBinding`，不重新解释 `.veac`。
