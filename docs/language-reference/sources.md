# Source

Item source 是闭合 Domain value。每个 variant 只拥有与自身有意义的 operand；不存在
`source { type = "..." }` property bag。

## Media 与 Nested Sequence

```veac,fragment
item(key, item_enabled(), during(0s, 4s),
  source_media(footage), source_timing_native())

item(key, item_enabled(), during(0s, 2s),
  source_nested_sequence(intro), source_timing_native())
```

Media source 引用已创建并由 Project 持有的 Resource handle。Nested source 引用 Sequence handle；
freeze/lowering 会检查 owner、cycle、depth 与 source-time mapping。

## Generated Source

```veac,fragment
source_generated(generator_transparent())
source_generated(generator_silence())
source_generated(generator_solid(#112233ff))
source_generated(generator_gradient(gradient_linear(
  vector(0.0, 0.0), vector(1.0, 1.0),
  [gradient_stop(0%, #112233ff), gradient_stop(100%, #ffeeccff)]
)))
source_generated(vector_shape(
  geometry_rectangle(rect(0.0, 0.0, 1.0, 1.0)),
  paint_present(paint_solid(#ffffffff)), stroke_none()
))
```

Generated graphics、shape、gradient 和 silence 都是 typed source，不需要伪造素材文件。

## Multicam

```veac,fragment
let host_angle = multicam_angle(identifier("host"), host, 0s);
let guest_angle = multicam_angle(identifier("guest"), guest, 120ms);
let interview = multicam_group(
  identifier("interview"), multicam_sync(multicam_sync_audio(), host_angle),
  [host_angle, guest_angle]
);
source_multicam(interview, [
  multicam_switch(host_angle, during(0s, 4s)),
  multicam_switch(guest_angle, during(4s, 3s))
])
```

Switch 必须从零开始连续覆盖完整 clip-local duration；angle 必须属于同一 group，resource kind、sync
reference 与 offset 都在 freeze 前验证。

Text 与 caption 使用 `source_text` / `source_caption` 并携带 typed `TextStyle`。共同的 font、layout、
decoration 与 animation contract 见[Text、caption 与 audio](text-caption-audio.md)。完整 source family
示例见 [`examples/`](../../examples/)。
