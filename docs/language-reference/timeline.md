# Timeline 与 Source Time

Sequence 拥有 typed Layer，Layer 拥有 Item。Layer constructor 为 `video_layer`、`visual_layer`、
`audio_layer` 和 `caption_layer`；每个 constructor 都要求 key、z-order、placement、state 和 routing。

```veac,fragment
let shot = item(
  identifier("opening"), item_enabled(), during(4s, 6s),
  source_media(interview),
  source_timing_mapped(source_mapping(
    source_time_linear(12s, 1.0, 1, playback_forward()),
    frame_nearest(), out_of_range_strict()
  ))
).with_visual(picture_style());
let picture = video_layer(
  identifier("picture"), 0, placement_free(), state, track_routing_default()
).with_item(shot);
let timeline = sequence(
  identifier("main"), "采访",
  sequence_settings(canvas(1920px, 1080px), frame_rate(30, 1), 48000)
).with_layer(picture);
```

三个时间域保持分离：

- record time: `during(start, duration)` 在 owning Sequence 中的位置；
- clip time: 从零开始的 Item local time；
- source time: media、nested sequence 或 multicam source 被采样的位置。

`source_timing_native()` 保持 source 自然时钟。`source_timing_mapped()` 接受 linear 或 segmented
curve map、frame sampling policy 与 closed out-of-range policy。负方向显式使用
`playback_reverse()`；定格使用 `source_freeze_frame(resource, at)`；循环和保持边界从不隐式发生。

```veac,fragment
source_mapping(
  source_time_curve([
    source_time_segment(1s, 2s, 3s, segment_linear()),
    source_time_segment(1s, 3s, 5s, segment_linear())
  ]),
  frame_blend(), out_of_range_hold_last()
)
```

视觉、音频、mask、effect 与 text animation 的传统 keyframe channel 使用闭合
`*_constant`/`*_keyframes` value。需要可编程 random-access expression 时，root `animate`
declaration residualize 到同一 canonical `Animatable::Binding` 模型。

Template contract 属于 Item，Item ID 就是 slot identity：

```veac,fragment
item(...).with_template(template_contract(
  slot_text(), fill_fit_duration(), "主标题文本",
  source_duration_any(), template_text_editable()
))
```

完整 source-time、nested boundary 和 frame-policy 示例见
[`examples/timeline-source-time/main.veac`](../../examples/timeline-source-time/main.veac)。
