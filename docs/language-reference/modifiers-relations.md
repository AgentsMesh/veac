# Visual、Effect、Relation 与 Apply

Item 通过 typed owner method 组合 visual/audio style、mask、effect 和 template contract。Operation 的
numeric ID、operand type、effect/stage 与 lowering action 都来自闭合 stdlib registry；未知 effect 或
parameter 无法进入 Core。

## Visual Style

`visual_style(layout, surface, masks, color_pipeline)` 保持原语边界：layout 组合 placement、frame fit
和 2D transform；surface 组合 opacity、z-order、blend 与 optional card；mask 是 closed shape；color 是
ordered pipeline。Transform 中 position/scale/rotation/crop 使用 typed `Animatable<T>`，可来自 constant、
keyframes 或 authored `animate` binding。

```veac,fragment
item(...).with_visual(visual_style(
  visual_layout(
    placement_anchor(anchor_center(), vector(0.0, 0.0)),
    frame_sized(1280px, 720px, fit_contain()),
    transform_2d(
      transform_motion(
        point_constant(point(0px, 0px)),
        vector_constant(vector(1.0, 1.0)), angle_constant(0deg)
      ),
      transform_geometry(
        vector(0.0, 0.0), flip_none(), vector(0.5, 0.5), crop_none()
      )
    )
  ),
  visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()),
  [], color_pipeline_none()
))
```

## Item Effect

Effects 按 `.with_effect(...)` 调用顺序执行：

```veac,fragment
item(...)
  .with_effect(video_blur_effect(
    identifier("blur"), effect_enabled(effect_window_full()), length_constant(8px)
  ))
  .with_effect(video_color_adjust_effect(
    identifier("grade"), effect_enabled(effect_window_full()),
    scalar_constant(0.1), scalar_constant(1.2), scalar_constant(0.9)
  ))
```

插件 Effect 不接受动态名字或参数 map；源码只能选择已固定版本的 typed descriptor：

```veac,fragment
item(...).with_effect(video_plugin_scalar_effect(
  identifier("monochrome"), effect_enabled(effect_window_full()),
  plugin_reference_monochrome_v1(), scalar_constant(0.75)
))
```

descriptor 是 static topology，并固定 schema、实现 identity、typed parameter schema、determinism、
支持的 backend 与内容 digest。缺少精确 backend adapter、digest 不匹配、遗漏或增加参数都会在 canonical
validation 或 render-plan preflight 中 fail closed；backend filter 字符串不会进入 `.veac` 或 Core。

## Relation

Transition、matte、sidechain、group 和 AV link 是 Sequence-owned relation：

```veac,fragment
timeline
  .with_relation(relation_transition(
    identifier("cut"), first, second, transition_dissolve(400ms)
  ))
  .with_relation(relation_group(identifier("edit-unit"), [picture, sound]))
```

Transition 必须是相邻真实 visual stream 的 centered true overlap；authored duration 必须等于交集，
两端都完整覆盖窗口。backend 禁止用 held-frame endpoint padding 伪造 handle。

## Apply

`apply` 直接表达 CompositeBand、Layer 或 ItemSet 作用域，不创建 adjustment clip：

```veac,fragment
apply(
  identifier("global-grade"), apply_enabled(), during(0s, 8s),
  apply_target_items([first, second]),
  [apply_effect_stage(
    identifier("contrast"), apply_stage_enabled(apply_stage_window_full()),
    video_color_adjust_effect(
      identifier("adjust"), effect_enabled(effect_window_full()),
      scalar_constant(0.0), scalar_constant(1.2), scalar_constant(1.0)
    )
  )],
  apply_mix(percent_constant(90%), blend_soft_light(), [])
)
```

Stage order、record range、mix、mask 和 target 都是 typed operands。完整示例见
[`examples/video-effects`](../../examples/video-effects)、
[`examples/masks-and-mattes`](../../examples/masks-and-mattes)、
[`examples/transitions`](../../examples/transitions) 和
[`examples/apply-scopes`](../../examples/apply-scopes)。
