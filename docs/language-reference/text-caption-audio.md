# Text、Caption 与 Audio

Text source 由 content 和 typed `TextStyle` 组成。Style 明确组合 metrics、layout、path、decoration、
rich spans 与 unit animation：

```veac,fragment
let style = text_style(
  text_metrics(
    font_stack(font_resource_ref(font), []),
    weight_bold(), font_style_normal(), 48px, 1px, 1.2, #ffffffff
  ),
  text_layout(
    text_box_width(900px), text_wrap_word(), text_overflow_ellipsis(),
    text_align_center(), text_align_middle(),
    writing_horizontal_tb(), orientation_mixed()
  ),
  text_path_none(),
  text_decoration(
    text_background_present(#07131dcc, 12px),
    text_outline_present(#22d3eeff, 2px), shadow_none()
  ),
  [], text_animation_none()
);
source_text("类型化文本", style)
```

Font stack 持有 Project-owned font Resource reference。Layout 覆盖 fixed/width/height box、wrap、
overflow、horizontal/vertical alignment、writing mode、glyph orientation 和 typed path。Rich span 使用
scalar index 的非重叠半开区间。Unit animation 支持 whole、line、word、grapheme 的 reveal、highlight、
opacity、position、scale、rotation 和 stagger。

Caption 复用同一个 TextStyle，并可携带 speaker：

```veac,fragment
source_caption("可烧录也可独立交付", style)
source_caption_speaker("欢迎", "讲述者", style)
```

Caption source 只能进入 caption Layer。`deliverable_caption_sidecar` 选择 typed caption Layer handle，
cue timing 来自 Item record span；SRT、WebVTT 和 ASS 是闭合 sidecar format。

Audio Item 使用 `AudioStyle`，不是字段袋：

```veac,fragment
audio_style(
  scalar_constant(0.8), scalar_constant(0.0),
  audio_playback(false, false, pitch_preserve()),
  [
    audio_high_pass(identifier("rumble"), 80.0, 0.7, 2),
    audio_compressor(
      identifier("voice"),
      compressor_settings(-18.0, 3.0, 10.0, 120.0, 4.0, 2.0, 75%)
    )
  ],
  audio_crossfade_present(200ms, 200ms, audio_fade_equal_power())
)
```

Gain/pan 可为 keyframes 或 authored temporal binding。Processor 顺序是语义；EQ、high/low pass、
compressor、limiter、gate、loudness、normalize、denoise 等 operation 各自有闭合参数。Audio Layer 可
route 到 typed bus，sidechain relation 使用 Layer/Item handle，不用字符串 lookup。

完整示例见 [`examples/text-layout`](../../examples/text-layout)、
[`examples/text-animation`](../../examples/text-animation)、
[`examples/captions-and-sidecars`](../../examples/captions-and-sidecars) 和
[`examples/audio-processing`](../../examples/audio-processing)。
