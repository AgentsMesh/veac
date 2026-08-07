# Project, Timeline, And Resources

## DomainTypes

```text
Context, ProjectSettings, SequenceSettings
Project, Sequence, Layer, Item
PlacementMode, TrackState, TrackPlayback, TrackAudioState, TrackIsolation, TrackEditing
TrackRouting, ItemState
Resource, ResourceLocation, StreamChoice, StreamIntent
```

`Project`, `Sequence`, `Layer`, and `Item` are opaque graph containers. `Resource` is an immutable,
Project-owned entity with one of six closed kinds. Host probe snapshots, metadata, revision, and
applied-operation history are intentionally absent from the authored surface.

## Settings And State

```veac
project_settings(timebase_hz: int) -> ProjectSettings
sequence_settings(canvas: Canvas, frame_rate: FrameRate, sample_rate_hz: int) -> SequenceSettings
placement_magnetic() -> PlacementMode
placement_free() -> PlacementMode
track_playback_enabled() -> TrackPlayback
track_playback_disabled() -> TrackPlayback
track_audio_audible() -> TrackAudioState
track_audio_muted() -> TrackAudioState
track_isolation_normal() -> TrackIsolation
track_isolation_solo() -> TrackIsolation
track_editing_unlocked() -> TrackEditing
track_editing_locked() -> TrackEditing
track_state(playback: TrackPlayback, audio: TrackAudioState, isolation: TrackIsolation,
            editing: TrackEditing) -> TrackState
track_routing_default() -> TrackRouting
track_routing_bus(bus: AudioBus) -> TrackRouting
item_enabled() -> ItemState
item_disabled() -> ItemState
```

Timebase and sample rate are positive. Solo and mute remain separate semantic axes. Non-audio layers
must use `track_audio_audible` and `track_routing_default`; canonical lowering materializes their
neutral state rather than silently discarding authored contradictory values.

## Resource Values

```veac
resource_file(path: text) -> ResourceLocation
resource_remote_http(url: text) -> ResourceLocation
stream_auto() -> StreamChoice
stream_disabled() -> StreamChoice
stream_global(index: int) -> StreamChoice
stream_intent(video: StreamChoice, audio: StreamChoice) -> StreamIntent
video_resource(key: identifier, location: ResourceLocation, identity: ContentIdentity,
               streams: StreamIntent) -> Resource
audio_resource(key: identifier, location: ResourceLocation, identity: ContentIdentity,
               stream: StreamChoice) -> Resource
image_resource(key: identifier, location: ResourceLocation, identity: ContentIdentity) -> Resource
font_resource(key: identifier, location: ResourceLocation, identity: ContentIdentity) -> Resource
lut1d_resource(key: identifier, location: ResourceLocation, identity: ContentIdentity) -> Resource
lut3d_resource(key: identifier, location: ResourceLocation, identity: ContentIdentity) -> Resource
```

Remote locations accept only absolute HTTP(S) URLs; file locations are module-relative normalized
paths. Every v6 resource has SHA-256 identity. Global stream indexes are non-negative. An audio
resource lowers with video disabled; image, font, and LUT resources lower with both streams disabled.
Video stream intent must select at least one media stream. Probe and selected type-index data are
derived by the pinned host probe and cannot be authored.

## Graph Construction

These operations are `GraphEmit`:

```veac
project(key: identifier, settings: ProjectSettings) -> Project
sequence(key: identifier, name: text, settings: SequenceSettings) -> Sequence
video_layer(key: identifier, order: int, placement: PlacementMode, state: TrackState,
            routing: TrackRouting) -> Layer
audio_layer(key: identifier, order: int, placement: PlacementMode, state: TrackState,
            routing: TrackRouting) -> Layer
visual_layer(key: identifier, order: int, placement: PlacementMode, state: TrackState,
             routing: TrackRouting) -> Layer
caption_layer(key: identifier, order: int, placement: PlacementMode, state: TrackState,
              routing: TrackRouting) -> Layer
item(key: identifier, state: ItemState, record: TimeRange, source: Source,
     timing: SourceTiming) -> Item
Project.with_resource(resource: Resource) -> Project
Project.with_sequence(sequence: Sequence) -> Project
Project.entry(sequence: Sequence) -> Project
Sequence.with_layer(layer: Layer) -> Sequence
Layer.with_item(item: Item) -> Layer
```

Layer kind is selected by its constructor, never by text. Child keys are unique within their owner;
order is a signed stable stacking/order value, not identity. `Project.entry` references an already
owned sequence and may be called exactly once. Every item always has explicit state, record range,
source, and source-timing intent; defaults are ordinary reusable functions, not hidden property bags.

## Atomic Plural Ownership

These `GraphEmit` methods attach an ordered homogeneous list as one transaction. They preserve input
order, accept an empty list as an identity update, and validate every child before changing ownership:

```veac
Project.with_resources(resources: list<Resource>) -> Project
Project.with_sequences(sequences: list<Sequence>) -> Project
Project.with_multicam_groups(groups: list<MulticamGroup>) -> Project
Project.with_annotations(annotations: list<Annotation>) -> Project
Project.with_deliveries(deliveries: list<Delivery>) -> Project
Sequence.with_layers(layers: list<Layer>) -> Sequence
Sequence.with_relations(relations: list<Relation>) -> Sequence
Sequence.with_applies(applies: list<Apply>) -> Sequence
Layer.with_items(items: list<Item>) -> Layer
```

Plural ownership is not a topology escape hatch. Duplicate children, duplicate sibling keys, stale
handles, already-owned entities, and cross-graph handles fail the whole call without partial attachment.
