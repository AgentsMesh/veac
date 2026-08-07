# Sources, Time Mapping, And Generators

## DomainTypes

```text
Source, SourceTiming, SourceMapping, SourceTimeMap, SourceTimeSegment
PlaybackDirection, FrameSynthesis, OutOfRangePolicy, SegmentInterpolation
Generator, Gradient, GradientStop, Paint, PaintChoice, StrokeChoice
VectorGeometry, PathCommand
```

`Source` is a closed description referenced by an Item. Resource, Sequence, and Multicam handles are
typed non-owning references; constructing a source never changes their ownership.

## Source Variants

```veac
source_media(resource: Resource) -> Source
source_freeze_frame(resource: Resource, source_at: time) -> Source
source_nested_sequence(sequence: Sequence) -> Source
source_multicam(group: MulticamGroup, switches: list<MulticamSwitch>) -> Source
source_text(content: text, style: TextStyle) -> Source
source_caption(content: text, style: TextStyle) -> Source
source_caption_speaker(content: text, speaker: text, style: TextStyle) -> Source
source_generated(generator: Generator) -> Source
```

Media accepts only video, audio, or image resources; freeze accepts video or image. Nested Sequence
and Multicam references must belong to the same Project graph. Multicam switches form a contiguous,
non-overlapping partition of the Item record duration.

## Source Time

```veac
source_timing_native() -> SourceTiming
source_timing_mapped(mapping: SourceMapping) -> SourceTiming
playback_forward() -> PlaybackDirection
playback_reverse() -> PlaybackDirection
frame_nearest() -> FrameSynthesis
frame_blend() -> FrameSynthesis
frame_motion_compensated() -> FrameSynthesis
out_of_range_strict() -> OutOfRangePolicy
out_of_range_hold_first() -> OutOfRangePolicy
out_of_range_hold_last() -> OutOfRangePolicy
out_of_range_hold_both() -> OutOfRangePolicy
segment_linear() -> SegmentInterpolation
segment_hold() -> SegmentInterpolation
source_time_linear(source_start: time, rate: scalar, repeat: int,
                   direction: PlaybackDirection) -> SourceTimeMap
source_time_segment(record_duration: time, source_start: time, source_end: time,
                    interpolation: SegmentInterpolation) -> SourceTimeSegment
source_time_curve(segments: list<SourceTimeSegment>) -> SourceTimeMap
source_mapping(map: SourceTimeMap, synthesis: FrameSynthesis,
               out_of_range: OutOfRangePolicy) -> SourceMapping
```

Rate is positive; direction carries reversal. Repeat is in `[1, 1024]`. Curve segments are non-empty,
positive-duration, ordered by list position, and contiguous on the record clock. Native timing is
valid only for sources whose canonical semantics do not require mapping. Frame synthesis and range
policy are explicit even when their reusable helper chooses nearest/strict.

## Gradients And Paint

```veac
gradient_stop(offset: percent, value: color) -> GradientStop
gradient_linear(start: Vector, end: Vector, stops: list<GradientStop>) -> Gradient
gradient_radial(center: Vector, radius: scalar, stops: list<GradientStop>) -> Gradient
paint_solid(value: color) -> Paint
paint_gradient(value: Gradient) -> Paint
paint_none() -> PaintChoice
paint_present(value: Paint) -> PaintChoice
stroke_none() -> StrokeChoice
stroke_present(paint: Paint, width: length) -> StrokeChoice
```

Stops contain at least two entries, are ordered by offset, and include only finite values. Radius and
stroke width are non-negative. `PaintChoice` and `StrokeChoice` are closed options, not nullable fields.

## Vector Geometry

```veac
geometry_rectangle(bounds: Rect) -> VectorGeometry
geometry_ellipse(bounds: Rect) -> VectorGeometry
geometry_rounded_rectangle(bounds: Rect, radius: scalar) -> VectorGeometry
geometry_polygon(points: list<Vector>) -> VectorGeometry
path_move_to(point: Vector) -> PathCommand
path_line_to(point: Vector) -> PathCommand
path_close() -> PathCommand
geometry_path(commands: list<PathCommand>) -> VectorGeometry
vector_shape(geometry: VectorGeometry, fill: PaintChoice,
             stroke: StrokeChoice) -> Generator
```

Polygons have at least three points. Paths begin with move-to and contain a drawable segment. Rounded
radius is non-negative. Fill and stroke are selected as whole variants, so the API cannot express an
incoherent half-stroke.

## Generator Variants

```veac
generator_solid(value: color) -> Generator
generator_gradient(value: Gradient) -> Generator
generator_transparent() -> Generator
generator_silence() -> Generator
```

`vector_shape` is the unique shape generator constructor. Silence is legal only on an audio Item;
visual generators are legal only on video or visual Items. The verifier enforces that owner/source
compatibility before any canonical graph is published.
