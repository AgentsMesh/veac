# Project 与 Resource

`main(Context) -> Project` 构造一个有根、连通、单 owner 的 graph。Project key、Sequence key、Layer
key 和 Item key 都是 typed `identifier` value，不是 JSON field name。

```veac,fragment
let footage = video_resource(
  identifier("interview"), resource_file("assets/interview.mov"),
  sha256("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
  stream_intent(stream_auto(), stream_auto())
);
let timeline = sequence(
  identifier("main"), "主时间线",
  sequence_settings(canvas(1920px, 1080px), frame_rate(30000, 1001), 48000)
);
project(identifier("documentary"), project_settings(600))
  .with_resource(footage)
  .with_sequence(timeline)
  .entry(timeline)
```

Resource constructor 是闭合的：`video_resource`、`audio_resource`、`image_resource`、
`font_resource`、`lut1d_resource` 和 `lut3d_resource`。file locator 与 SHA-256 content identity 是
typed operand；video/audio stream intent 使用 `stream_auto()`、`stream_disabled()` 或显式 closed
selection。不存在任意 locator map 或可透传 metadata bag。

Project 通过 owner method 接入 resource、sequence、multicam group、annotation 和 delivery。一个
handle 只能属于创建它的 graph，entity 只能接入一个 owner，重复 key 和 cycle 都使 transaction
回滚。`.entry(timeline)` 接受已连接的 Sequence handle，不接受字符串引用。

canonical lowering 稳定派生 `prj_`、`med_`、`seq_`、`trk_`、`itm_`、`fx_`、`rel_` 和 `out_`
等 ID。源码编辑稳定 key 和构造调用，不直接编辑这些派生 ID。背景是可见内容，应由 generated/media
Item 表达，不是 Project setting。

当前 schema v10 不包含 Project、Material、Sequence 和 Clip 上的任意 `metadata` map，并把效果实例收口为闭合的类型化变体。可执行来源只进入闭合的
typed `authorship`：operation 使用 current opset 的 numeric opcode，event/definition kind 是 enum，
owner child provenance 使用 typed ID entry 数组。数组必须排序且与实际 owner graph 精确一致；source、
span、logical path、call stack、iteration 与总字节数均有 validation budget。program identity 只存在于
`ProjectEnvelope.executable`，不会在 authorship 中复制一份可能漂移的字符串键对象。

## 类型化效果 ABI

canonical `EffectInstance` 只保存稳定 ID、启用状态、可选窗口和一个 closed `Effect` variant。颜色调整、
模糊、方向性模糊、锐化、暗角、颗粒、色度键、亮度键、溢色抑制、稳定、响度归一化与已固定版本的单色插件各自拥有
确定字段；数值动画直接使用 `Animatable<f64>`，颜色、布尔值和静态 LUFS 使用对应具体类型。JSON 中
不存在 `effect_type` 加 `map<string, value>`，未知 variant、缺失字段、额外字段和类型不匹配都会在
反序列化边界失败。

源码编辑和 Temporal attachment 通过 closed `EffectParameter` selector 寻址现有动态叶子。selector
只能选择该 variant 已拥有的字段，不能创建参数、改变字段类型或生成新的 topology。版本化插件 variant
还携带 validated descriptor digest；registry、plan preflight 与 backend adapter 必须对同一 digest 达成
一致，否则 fail closed。

完整可执行示例见 [`examples/all-features/main.veac`](../../examples/all-features/main.veac)。
