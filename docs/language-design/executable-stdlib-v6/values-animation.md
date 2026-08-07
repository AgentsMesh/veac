# Shared Values And Animation

## DomainTypes

```text
Canvas, FrameRate, TimeRange, Point, Vector, Rect, ContentIdentity
Interpolation
ScalarAnimation, ScalarKeyframe
LengthAnimation, LengthKeyframe
PercentAnimation, PercentKeyframe
AngleAnimation, AngleKeyframe
PointAnimation, PointKeyframe
VectorAnimation, VectorKeyframe
RectAnimation, RectKeyframe
```

The only scalar primitives used below are `int`, `scalar`, `time`, `length`, `percent`, `angle`,
`text`, `color`, `bool`, and `identifier`. Unit-bearing values never travel as explanatory strings.
`percent` is normalized by the type system; validation accepts its documented closed range.

## Structural Values

```veac
canvas(width: length, height: length) -> Canvas
frame_rate(numerator: int, denominator: int) -> FrameRate
during(start: time, duration: time) -> TimeRange
point(x: length, y: length) -> Point
vector(x: scalar, y: scalar) -> Vector
rect(x: scalar, y: scalar, width: scalar, height: scalar) -> Rect
sha256(digest: text) -> ContentIdentity
```

Frame-rate numerator and denominator are positive. Ranges are half-open and duration is non-negative.
`sha256` accepts exactly 64 lowercase hexadecimal digits; no alternate identity algorithm is in v6.

## Interpolation

```veac
interpolation_hold() -> Interpolation
interpolation_linear() -> Interpolation
interpolation_ease_in() -> Interpolation
interpolation_ease_out() -> Interpolation
interpolation_ease_in_out() -> Interpolation
interpolation_spring(frequency: scalar, decay: scalar, initial_velocity: scalar) -> Interpolation
interpolation_cubic_bezier(x1: scalar, y1: scalar, x2: scalar, y2: scalar) -> Interpolation
```

Spring frequency and decay are positive. Cubic Bezier x coordinates are in `[0, 1]`. Interpolation on
a key applies until the next key; the final key's interpolation is retained but never sampled.

## Scalar Animation

```veac
scalar_constant(value: scalar) -> ScalarAnimation
scalar_keyframe(key: identifier, at: time, value: scalar, easing: Interpolation) -> ScalarKeyframe
scalar_keyframes(values: list<ScalarKeyframe>) -> ScalarAnimation
```

## Length Animation

```veac
length_constant(value: length) -> LengthAnimation
length_keyframe(key: identifier, at: time, value: length, easing: Interpolation) -> LengthKeyframe
length_keyframes(values: list<LengthKeyframe>) -> LengthAnimation
```

## Percent Animation

```veac
percent_constant(value: percent) -> PercentAnimation
percent_keyframe(key: identifier, at: time, value: percent, easing: Interpolation) -> PercentKeyframe
percent_keyframes(values: list<PercentKeyframe>) -> PercentAnimation
```

## Angle Animation

```veac
angle_constant(value: angle) -> AngleAnimation
angle_keyframe(key: identifier, at: time, value: angle, easing: Interpolation) -> AngleKeyframe
angle_keyframes(values: list<AngleKeyframe>) -> AngleAnimation
```

## Point Animation

```veac
point_constant(value: Point) -> PointAnimation
point_keyframe(key: identifier, at: time, value: Point, easing: Interpolation) -> PointKeyframe
point_keyframes(values: list<PointKeyframe>) -> PointAnimation
```

## Vector Animation

```veac
vector_constant(value: Vector) -> VectorAnimation
vector_keyframe(key: identifier, at: time, value: Vector, easing: Interpolation) -> VectorKeyframe
vector_keyframes(values: list<VectorKeyframe>) -> VectorAnimation
```

## Rectangle Animation

```veac
rect_constant(value: Rect) -> RectAnimation
rect_keyframe(key: identifier, at: time, value: Rect, easing: Interpolation) -> RectKeyframe
rect_keyframes(values: list<RectKeyframe>) -> RectAnimation
```

Every keyframe list is non-empty, strictly increasing by `at`, and unique by `key`. All values are
finite and keep one unit kind across a length or point curve. G2 lowers constants and keyframes only.
G3 may bind the same typed animation leaf to a residual temporal program; v6 never accepts an
expression string or a generic `Animatable<T>` runtime value.
