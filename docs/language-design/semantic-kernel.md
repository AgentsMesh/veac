# VEAC Semantic Kernel

The semantic kernel is the closed model shared by authoring, canonical IR,
planning, and execution:

```text
.veac -> parser -> typed AST -> lowering -> canonical JSON IR
      -> planner -> backend artifact -> runtime

canonical JSON IR + JSON EditBatch -> atomic edit -> canonical JSON IR
```

The edit path neither defines a textual edit DSL nor rewrites `.veac`.

## Authoring And Canonical Ownership

```text
Authoring project                 Canonical Project
|- settings/resources            |- settings/materials
|- multicam groups               |- multicam_groups
|- annotations                   |- annotations and relations
|- sequences                     |- sequences
|  |- layers                     |  |- tracks
|  |  `- items                   |  |  `- clips
|  |- transitions/relations      |  |- transitions
|  `- scoped Apply               |  `- applies
`- deliveries                   `- render_configs/deliverables
```

Layer/item are authoring names for canonical Track/Clip. Relations are authored
inside a sequence and normalized into the project relation collection.
Multicam, annotation, and delivery declarations are project members. A delivery
or multicam group is never owned by a sequence.

## Closed Authoring Sources

An item owns exactly one source from this six-variant union:

1. Media resource reference.
2. Styled text.
3. Caption cue.
4. Generated transparent, silence, solid, gradient, or shape content.
5. Nested sequence reference.
6. Multicam group reference with item-local angle switches.

Unknown tags fail parsing. Solid color is generated media, not `source color`.
Caption sidecars are artifacts, not sources. Canonical `ClipSource` additionally
contains `FreezeFrame`, produced only by lowering media plus `mapping freeze`.

## Timeline Kernel

Every item separates its sequence placement from its source sampling:

- `record_range` determines where it appears.
- the closed source union determines what it contains.
- an optional mapping determines source time.
- typed visual/audio/text properties determine presentation.

Only media and sequence sources accept mappings. Authoring mappings are linear,
freeze, or curve. Lowering maps linear to a linear time map and curve keys to
ordered segments. Media freeze becomes `ClipSource::FreezeFrame` with no source
mapping; sequence freeze becomes one hold segment. Curve interpolation is only
`linear` or `hold`. Linear and curve use `strict`, `hold-first`, `hold-last`, or
`hold-both`; freeze has no outside policy.

All record times become exact integer ticks. Validation rejects gaps, invalid
key order, illegal endpoint coverage, overflow, and mismatched durations.

## Composition Kernel

Composition is explicit:

- Layers establish deterministic stacking and timing.
- Transitions consume an exact cut with typed overlap/alignment.
- Relations express typed constraints across items.
- Apply targets a composite band, a layer, or an exact item set.

Apply owns an ordered stage list plus typed mix, opacity/blend, mask, and
optional matte-consumer semantics. It is not a synthetic clip or generic
property map. Target membership and active intervals resolve before codegen.

Visual item rendering has one canonical order:

```text
source -> content geometry/effects/masks -> shadow split -> placement
       -> clip track matte -> item apply -> final composition
```

Track matte and item apply process both the shadow and foreground branches.
Final composition uses normal source-over for shadow and the authored blend
mode for foreground. Matte-source and transition-endpoint rendering flatten the
processed branches with normal source-over before those consumers use them.

## Color And Effects Kernel

A pipeline preserves stage order. Color stages include typed primary controls,
curves, wheels, matrices, color-space transforms, tone maps, and LUTs. A LUT
stage references a project `lut-1d` or `lut-3d` material. LUT1D permits
`nearest`, `linear`, `cosine`, `cubic`, and `spline`; LUT3D permits `nearest`,
`trilinear`, `tetrahedral`, `pyramid`, and `prism`.

Video effects, masks, transforms, mattes, opacity, and blend modes are closed
unions. Unsupported variants fail before backend generation; raw FFmpeg filter
text is not a language primitive.

## Media And Streams Kernel

Authoring resources are closed to video, audio, image, font, LUT1D, and LUT3D.
Locations are `local` paths or pinned `remote` URIs. Video and audio selection
are independently `auto` or `disabled` and lower to canonical stream intent.
Probe normalization records exact stream selections and media facts separately.
Planning does not rerun authoring selection heuristics.

Media bytes, provider credentials, and secrets stay outside source and IR.

## Audio Kernel

```text
source -> item gain/pan -> layer processor chain -> bus route -> master mix
```

An audio layer may route to a bus ID. Referencing a route establishes bus
identity; no standalone bus declaration exists. Each audio-stem artifact selects
one source: project master, one track, or one routed bus. Processor order and
route identity remain typed through planning.

## Caption Kernel

A caption item carries text plus typed language, speaker, confidence, style,
karaoke timing, and word timing. A caption-sidecar artifact selects SRT, WebVTT,
or ASS plus caption track IDs. Its containing delivery selects the sequence.
Sidecar generation is separate from burning text into video.

## Delivery Kernel

Each project-owned delivery selects a sequence, optional raster contract, and a
non-empty artifact collection. Artifacts form a closed union: video, image
sequence, caption sidecar, audio stem, scope, audio file, animated image, still
image, and adaptive package.

Artifact bodies are typed recipes, not anonymous settings. `mux` owns container
and stream recipes; `encode` owns codec settings; `source`, `frame`, `canvas`,
`numbering`, `analyze`, and `package` remain separate domain primitives. Lowering
creates one schema-v5 `RenderConfig` and one canonical `Deliverable` per artifact.
Canonical settings use tagged enums and reject unknown fields.

## Edit Kernel

An EditBatch is canonical JSON containing required `operation_id`,
`base_revision`, and `atomic` fields plus ordered `preconditions` and tagged
`operations`. Operations are a closed union
covering Insert, Set, Move, Remove, locks, and typed convenience setters.

The editor checks revision, preconditions, and locks; rewrites typed references
where defined; validates the complete project; and commits or rolls back the
entire batch. It never applies a valid prefix of an invalid batch.

## Determinism Rules

- IDs are stable and unique in their typed owning namespace.
- Ordered declarations remain ordered where order is semantic.
- Time is exact under the project timebase.
- Unknown variants and fields fail closed.
- Formatter output is idempotent.
- Lowering emits only canonical schema version 5 constructs.
- Planning consumes canonical IR, never authoring syntax.
- Backend artifacts contain no unresolved authoring choices.
