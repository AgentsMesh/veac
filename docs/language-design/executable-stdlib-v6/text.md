# Text And Captions

## DomainTypes

```text
TextStyle, FontRef, FontStack, FontWeight, FontStyle, TextMetrics
TextBox, TextWrap, TextOverflow, HorizontalTextAlignment, VerticalTextAlignment
TextWritingMode, TextOrientation, TextLayout
TextPath, TextPathChoice, TextPathAlignment
TextBackgroundChoice, TextOutlineChoice, TextDecoration
TextRunStyle, TextSpan
TextAnimation, TextAnimationChoice, TextGranularity, TextUnitTransform
TextHighlightChoice
```

Text ranges use Unicode scalar indexes, never UTF-8 byte offsets. Shaping, fallback, bidirectional
ordering, and grapheme segmentation are deterministic services pinned by the runtime fingerprint.

## Typography

```veac
font_family(name: text) -> FontRef
font_resource_ref(resource: Resource) -> FontRef
font_stack(primary: FontRef, fallbacks: list<FontRef>) -> FontStack
weight_thin() -> FontWeight
weight_extra_light() -> FontWeight
weight_light() -> FontWeight
weight_normal() -> FontWeight
weight_medium() -> FontWeight
weight_semi_bold() -> FontWeight
weight_bold() -> FontWeight
weight_extra_bold() -> FontWeight
weight_black() -> FontWeight
font_style_normal() -> FontStyle
font_style_italic() -> FontStyle
font_style_oblique() -> FontStyle
text_metrics(fonts: FontStack, weight: FontWeight, style: FontStyle, size: length,
             tracking: length, line_height: scalar, fill: color) -> TextMetrics
```

Material font references accept only Font Resources. Size and line height are positive; fallback order
is semantic and duplicate references are rejected.

## Layout

```veac
text_box_auto() -> TextBox
text_box_width(width: length) -> TextBox
text_box_height(height: length) -> TextBox
text_box_fixed(width: length, height: length) -> TextBox
text_wrap_none() -> TextWrap
text_wrap_word() -> TextWrap
text_wrap_character() -> TextWrap
text_overflow_visible() -> TextOverflow
text_overflow_clip() -> TextOverflow
text_overflow_ellipsis() -> TextOverflow
text_align_left() -> HorizontalTextAlignment
text_align_center() -> HorizontalTextAlignment
text_align_right() -> HorizontalTextAlignment
text_align_top() -> VerticalTextAlignment
text_align_middle() -> VerticalTextAlignment
text_align_bottom() -> VerticalTextAlignment
writing_horizontal_tb() -> TextWritingMode
writing_vertical_rl() -> TextWritingMode
writing_vertical_lr() -> TextWritingMode
orientation_mixed() -> TextOrientation
orientation_upright() -> TextOrientation
orientation_sideways() -> TextOrientation
text_layout(box: TextBox, wrap: TextWrap, overflow: TextOverflow,
            horizontal: HorizontalTextAlignment, vertical: VerticalTextAlignment,
            writing: TextWritingMode, orientation: TextOrientation) -> TextLayout
```

Box dimensions are positive. Ellipsis requires a bounded axis. Orientation follows the logical writing
mode and is not encoded as a backend transform.

## Path And Decoration

```veac
text_path_align_start() -> TextPathAlignment
text_path_align_center() -> TextPathAlignment
text_path_align_end() -> TextPathAlignment
text_path(points: list<Point>, start_offset: length, reverse: bool,
          alignment: TextPathAlignment) -> TextPath
text_path_none() -> TextPathChoice
text_path_present(value: TextPath) -> TextPathChoice
text_background_none() -> TextBackgroundChoice
text_background_present(fill: color, padding: length) -> TextBackgroundChoice
text_outline_none() -> TextOutlineChoice
text_outline_present(value: color, width: length) -> TextOutlineChoice
text_decoration(background: TextBackgroundChoice, outline: TextOutlineChoice,
                shadow: ShadowChoice) -> TextDecoration
```

A path contains at least two points. Padding and outline width are non-negative. Text shadow reuses the
closed visual Shadow value and keeps the same physical-unit semantics.

## Rich Spans

```veac
text_run_style(font: FontRef, weight: FontWeight, style: FontStyle,
               size: length, fill: color) -> TextRunStyle
text_span(start: int, end: int, style: TextRunStyle) -> TextSpan
```

Span ranges are half-open, non-empty, in bounds, sorted, and non-overlapping. `TextRunStyle` is a full
override value; lowering materializes its five attributes and never accepts a sparse field bag.

## Unit Animation

```veac
text_whole() -> TextGranularity
text_line() -> TextGranularity
text_word() -> TextGranularity
text_grapheme() -> TextGranularity
text_unit_transform(position: PointAnimation, scale: VectorAnimation,
                    rotation: AngleAnimation) -> TextUnitTransform
text_highlight_none() -> TextHighlightChoice
text_highlight_present(fill: color, progress: PercentAnimation) -> TextHighlightChoice
text_animation(granularity: TextGranularity, transform: TextUnitTransform,
               reveal: PercentAnimation, highlight: TextHighlightChoice,
               opacity: PercentAnimation, stagger: time) -> TextAnimation
text_animation_none() -> TextAnimationChoice
text_animation_present(value: TextAnimation) -> TextAnimationChoice
```

Stagger is non-negative. Unit N samples transform and opacity at clip time minus `N * stagger`.
Reveal and highlight progress use logical reading order.

## Complete Style And Sources

```veac
text_style(metrics: TextMetrics, layout: TextLayout, path: TextPathChoice,
           decoration: TextDecoration, spans: list<TextSpan>,
           animation: TextAnimationChoice) -> TextStyle
source_text(content: text, style: TextStyle) -> Source
source_caption(content: text, style: TextStyle) -> Source
source_caption_speaker(content: text, speaker: text, style: TextStyle) -> Source
```

These source signatures are the same operations cataloged in `sources-generators.md`, repeated here to
make the text family closed. Empty speaker is invalid; absence uses the distinct caption constructor.
