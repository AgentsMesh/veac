# Relations And Adjustment Applies

## DomainTypes

```text
Transition, FadeColor, CardinalDirection, ZoomDirection, CircleDirection
MatteMode, SidechainSettings, SidechainWindow, Relation
Apply, ApplyTarget, ApplyState, ApplyStage, ApplyStageState, ApplyStageWindow
ApplyMix
```

Relations accept exact endpoint types rather than a generic endpoint union. All relation and Apply
keys are stable within the owning Sequence.

## Centered Transitions

```veac
fade_transparent() -> FadeColor
fade_black() -> FadeColor
fade_white() -> FadeColor
direction_left() -> CardinalDirection
direction_right() -> CardinalDirection
direction_up() -> CardinalDirection
direction_down() -> CardinalDirection
zoom_in() -> ZoomDirection
zoom_out() -> ZoomDirection
circle_open() -> CircleDirection
circle_close() -> CircleDirection
transition_dissolve(duration: time) -> Transition
transition_fade(duration: time, value: FadeColor) -> Transition
transition_wipe(duration: time, direction: CardinalDirection, angle: angle,
                softness: percent) -> Transition
transition_slide(duration: time, direction: CardinalDirection,
                 amount: scalar) -> Transition
transition_zoom(duration: time, direction: ZoomDirection, amount: scalar) -> Transition
transition_circle(duration: time, direction: CircleDirection,
                  softness: percent) -> Transition
transition_pixelize(duration: time, amount: percent) -> Transition
```

Every transition is centered; v6 exposes no alternative alignment. Duration is positive and must fit
true adjacent overlap. Slide/zoom amount is `[0.1,4]`; pixelize is greater than zero. The verifier and
planner preserve centered overlap rather than turning it into one-sided opacity envelopes.

## Relation Construction

```veac
matte_alpha() -> MatteMode
matte_luma() -> MatteMode
sidechain_window_full() -> SidechainWindow
sidechain_window_during(range: TimeRange) -> SidechainWindow
sidechain_settings(threshold_db: scalar, ratio: scalar, attack_ms: scalar,
                   release_ms: scalar, active: SidechainWindow) -> SidechainSettings
relation_transition(key: identifier, from: Item, to: Item,
                    transition: Transition) -> Relation
relation_matte_item(key: identifier, producer: Item, consumer: Item,
                    mode: MatteMode, invert: bool) -> Relation
relation_matte_apply(key: identifier, producer: Item, consumer: Apply,
                     mode: MatteMode, invert: bool) -> Relation
relation_sidechain_track(key: identifier, source: Layer, target: Item,
                         settings: SidechainSettings) -> Relation
relation_sidechain_bus(key: identifier, source: AudioBus, target: Item,
                       settings: SidechainSettings) -> Relation
relation_group(key: identifier, members: list<Item>) -> Relation
relation_av_link(key: identifier, video: Item, audio: list<Item>) -> Relation
Sequence.with_relation(relation: Relation) -> Sequence
```

Relation constructors and `Sequence.with_relation` are `GraphEmit`. Transition endpoints are visual
Items; sidechain source Layer and target Item are audio-capable. Group has at least two distinct Items;
AV link has at least one audio Item. Every endpoint belongs to the owning Sequence. Sidechain active
window is target-Item-relative; ratio is at least one and times are non-negative.

## Apply Target And State

```veac
apply_target_composite_band(from: Layer, through: Layer) -> ApplyTarget
apply_target_layer(layer: Layer) -> ApplyTarget
apply_target_items(items: list<Item>) -> ApplyTarget
apply_enabled() -> ApplyState
apply_disabled() -> ApplyState
apply_stage_window_full() -> ApplyStageWindow
apply_stage_window_during(range: TimeRange) -> ApplyStageWindow
apply_stage_enabled(window: ApplyStageWindow) -> ApplyStageState
apply_stage_disabled(window: ApplyStageWindow) -> ApplyStageState
```

Composite endpoints are in inclusive stack order. Item-set targets are non-empty, unique, and sorted
by stable logical key before lowering; source list order is provenance, not execution order.

## Apply Stages And Mix

```veac
apply_color_stage(key: identifier, state: ApplyStageState,
                  pipeline: ColorPipeline) -> ApplyStage
apply_effect_stage(key: identifier, state: ApplyStageState,
                   effect: Effect) -> ApplyStage
apply_mix(opacity: PercentAnimation, blend: BlendMode,
          masks: list<Mask>) -> ApplyMix
```

Stage call/list order is execution order. Stage windows are Apply-relative. An effect stage retains both
stage state and keyed effect state; both must be enabled for execution.

## Apply Graph Operations

```veac
apply(key: identifier, state: ApplyState, record: TimeRange, target: ApplyTarget,
      stages: list<ApplyStage>, mix: ApplyMix) -> Apply
Sequence.with_apply(apply: Apply) -> Sequence
```

Both operations are `GraphEmit`. An Apply has a non-empty stage list, a positive record duration, and a
target owned by the same Sequence. Mix executes after its ordered stages. Apply masks are local to that
mix and do not mutate masks on target Items.
