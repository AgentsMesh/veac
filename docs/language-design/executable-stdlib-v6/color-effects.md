# Color Pipelines And Effects

## DomainTypes

```text
ColorSpace, ColorPrimaries, ColorTransfer, ColorMatrix, ColorRange
ColorPipeline, ColorStage, BasicColorAdjustment, RgbMatrixAdjustment, HslAdjustment, HueRange
ToneCurve, ToneCurveChannel, ToneInterpolation, CurvePoint, ColorCurves
ColorWheel, LiftGammaGain, LutInterpolation, LutApplication
Effect, EffectState, EffectWindow, PluginEffectDescriptor
```

## Color Space

```veac
primaries_bt709() -> ColorPrimaries
primaries_bt470m() -> ColorPrimaries
primaries_bt470bg() -> ColorPrimaries
primaries_smpte170m() -> ColorPrimaries
primaries_smpte240m() -> ColorPrimaries
primaries_film() -> ColorPrimaries
primaries_bt2020() -> ColorPrimaries
primaries_smpte428() -> ColorPrimaries
primaries_smpte431() -> ColorPrimaries
primaries_smpte432() -> ColorPrimaries
transfer_bt709() -> ColorTransfer
transfer_gamma22() -> ColorTransfer
transfer_gamma28() -> ColorTransfer
transfer_smpte170m() -> ColorTransfer
transfer_smpte240m() -> ColorTransfer
transfer_linear() -> ColorTransfer
transfer_srgb() -> ColorTransfer
transfer_bt2020_10() -> ColorTransfer
transfer_bt2020_12() -> ColorTransfer
transfer_smpte2084() -> ColorTransfer
transfer_arib_std_b67() -> ColorTransfer
matrix_rgb() -> ColorMatrix
matrix_bt709() -> ColorMatrix
matrix_fcc() -> ColorMatrix
matrix_bt470bg() -> ColorMatrix
matrix_smpte170m() -> ColorMatrix
matrix_smpte240m() -> ColorMatrix
matrix_ycgco() -> ColorMatrix
matrix_bt2020_ncl() -> ColorMatrix
range_limited() -> ColorRange
range_full() -> ColorRange
color_space(primaries: ColorPrimaries, transfer: ColorTransfer,
            matrix: ColorMatrix, range: ColorRange) -> ColorSpace
```

## Color Stages

```veac
basic_color(exposure_stops: scalar, temperature_kelvin: scalar, tint: scalar,
            highlights: scalar, shadows: scalar, fade: scalar) -> BasicColorAdjustment
rgb_matrix(coefficients: list<scalar>, offsets: list<scalar>) -> RgbMatrixAdjustment
hue_red() -> HueRange
hue_yellow() -> HueRange
hue_green() -> HueRange
hue_cyan() -> HueRange
hue_blue() -> HueRange
hue_magenta() -> HueRange
hsl_adjustment(range: HueRange, hue: angle, saturation: scalar,
               lightness: scalar) -> HslAdjustment
curve_point(input: percent, output: percent) -> CurvePoint
tone_natural() -> ToneInterpolation
tone_monotonic() -> ToneInterpolation
tone_curve(points: list<CurvePoint>, interpolation: ToneInterpolation) -> ToneCurve
tone_channel_none() -> ToneCurveChannel
tone_channel_present(curve: ToneCurve) -> ToneCurveChannel
color_curves(luma: ToneCurveChannel, red: ToneCurveChannel, green: ToneCurveChannel,
             blue: ToneCurveChannel) -> ColorCurves
color_wheel(red: scalar, green: scalar, blue: scalar) -> ColorWheel
lift_gamma_gain(lift: ColorWheel, gamma: ColorWheel, gain: ColorWheel) -> LiftGammaGain
```

The matrix has exactly nine row-major coefficients and three offsets. Curves have at least two points
with strictly increasing input. Channel absence is a closed value rather than nullable projection.

## LUT And Pipeline

```veac
lut_nearest() -> LutInterpolation
lut_linear() -> LutInterpolation
lut_cosine() -> LutInterpolation
lut_cubic() -> LutInterpolation
lut_spline() -> LutInterpolation
lut_trilinear() -> LutInterpolation
lut_tetrahedral() -> LutInterpolation
lut_pyramid() -> LutInterpolation
lut_prism() -> LutInterpolation
lut_application(resource: Resource, interpolation: LutInterpolation) -> LutApplication
color_stage_basic(value: BasicColorAdjustment) -> ColorStage
color_stage_matrix(value: RgbMatrixAdjustment) -> ColorStage
color_stage_hsl(value: HslAdjustment) -> ColorStage
color_stage_curves(value: ColorCurves) -> ColorStage
color_stage_wheels(value: LiftGammaGain) -> ColorStage
color_stage_lut(value: LutApplication) -> ColorStage
color_pipeline(input: ColorSpace, working: ColorSpace, output: ColorSpace,
               stages: list<ColorStage>) -> ColorPipeline
```

LUT application accepts only a LUT1D or LUT3D Resource. Stage list order is exact and retained.

## Effect State

```veac
effect_window_full() -> EffectWindow
effect_window_during(range: TimeRange) -> EffectWindow
effect_enabled(window: EffectWindow) -> EffectState
effect_disabled(window: EffectWindow) -> EffectState
```

An effect window is Item-relative and half-open. Disabled effects remain addressable for source edits.

## Eleven Built-Ins

```veac
video_color_adjust_effect(key: identifier, state: EffectState, brightness: ScalarAnimation,
                          contrast: ScalarAnimation, saturation: ScalarAnimation) -> Effect
video_blur_effect(key: identifier, state: EffectState, radius: LengthAnimation) -> Effect
video_sharpen_effect(key: identifier, state: EffectState, amount: ScalarAnimation) -> Effect
video_vignette_effect(key: identifier, state: EffectState, amount: PercentAnimation) -> Effect
video_grain_effect(key: identifier, state: EffectState, amount: PercentAnimation) -> Effect
video_chroma_key_effect(key: identifier, state: EffectState, key_color: color,
                        similarity: PercentAnimation, blend: PercentAnimation) -> Effect
video_luma_key_effect(key: identifier, state: EffectState, threshold: PercentAnimation,
                      tolerance: PercentAnimation, softness: PercentAnimation, invert: bool) -> Effect
video_chroma_spill_effect(key: identifier, state: EffectState, key_color: color,
                          amount: PercentAnimation, spill_range: PercentAnimation) -> Effect
video_stabilize_effect(key: identifier, state: EffectState, stabilization: bool) -> Effect
audio_normalize_effect(key: identifier, state: EffectState, target_lufs: scalar) -> Effect
Item.with_effect(effect: Effect) -> Item
```

`Item.with_effect` is `GraphEmit`; call order is chain order. Brightness is `[-1,1]`, contrast and
saturation `[0,4]`, blur `[0px,100px]`, directional blur angle `[0deg,360deg]` and radius
`[0px,100px]`, sharpen `[0,10]`, normalized parameters `[0%,100%]`, chroma similarity is greater
than zero, and target LUFS is `[-70,-5]`. Stabilization and target LUFS are static leaves. There is
no generic effect constructor and no plugin parameter map in v6.

## Versioned Plugin Descriptor

```veac
plugin_reference_monochrome_v1() -> PluginEffectDescriptor
video_plugin_scalar_effect(key: identifier, state: EffectState,
                           descriptor: PluginEffectDescriptor,
                           amount: ScalarAnimation) -> Effect
```

Plugin selection is static topology. The descriptor constructor resolves to a registry entry that pins
descriptor schema v1, implementation identity, the typed `amount: ScalarAnimation` schema and
`[0,1]` range, deterministic execution, the FFmpeg 8 backend contract, and a canonical SHA-256 digest.
The effect constructor accepts no names, maps, backend flags, or untyped values. A descriptor whose
digest or backend adapter is absent fails closed before rendering. The reference descriptor mixes the
input toward monochrome at `0` to `1`; its typed amount may remain a Temporal leaf.

## Directional Blur (v8)

The v8 signature is appended after the existing v7 operations to keep every published Core operation
identity stable while adding the eleventh built-in.

```veac
video_directional_blur_effect(key: identifier, state: EffectState, angle: AngleAnimation,
                              radius: LengthAnimation) -> Effect
```

The registry ranges are also runtime sink contracts. A Temporal result is clamped to angle
`[0deg,360deg]` and radius `[0px,100px]` immediately before the typed backend command is emitted;
the canonical Temporal program itself is unchanged and out-of-range values never reach FFmpeg.
Directional blur expands alpha in premultiplied space only while its half-open effect window is
active and the clamped radius is greater than zero. A constant zero radius emits no effect graph;
keyframed and Temporal radii use the same residualized positive-radius predicate for blur,
premultiply, and unpremultiply. Zero-radius frames and frames outside the window therefore preserve
straight-alpha samples exactly instead of round-tripping their color planes. Every visual clip enters
one `gbrap16le` working-format boundary after crop/frame conformance and before effects, scale, shear,
and rotation; a disabled effect can therefore neither introduce an earlier conversion nor change
downstream filter negotiation relative to an otherwise identical clip without that effect.
