# VEAC Semantic Kernel

The semantic kernel is the closed model shared by executable source, canonical IR,
planning, and execution:

```text
.veac source graph -> typed HIR -> verified Core v10
  -> graph execution/freeze + temporal residualization
  -> canonical JSON IR -> planner -> backend artifact -> runtime

SourceEditBatch + source graph -> atomic edit -> recompile
EditBatch + canonical JSON IR  -> atomic edit -> canonical JSON IR
```

root absolute animation 与 component owner-relative attachment 共用同一个 Temporal kernel。attachment
在 Build graph 中只携带 typed handle、closed selector 和 Pure closure；graph freeze 后才解析 absolute
sink，Temporal 只驱动已批准的 dynamic leaf，绝不决定 entity 数量、owner、顺序或 collection shape。

Compile-time modules, constants, functions and nominal values generate kernel facts;
declarations themselves never enter canonical IR. Neither edit path
decompiles canonical JSON into `.veac`.

## Domain And Canonical Ownership

```text
Executable Domain graph          Canonical Project
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

Layer/Item are Domain names for canonical Track/Clip. Relations attach to a Sequence and normalize into
the project relation collection. Multicam, annotation, and delivery values attach to Project. A delivery
or multicam group is never owned by a sequence.

## Closed Sources

An item owns exactly one source from this six-variant union:

1. Media resource reference.
2. Styled text.
3. Caption cue.
4. Generated transparent, silence, solid, gradient, or shape content.
5. Nested sequence reference.
6. Multicam group reference with item-local angle switches.

Unknown constructors fail resolution. Solid color is generated media, not an Item property.
Caption sidecars are artifacts, not sources. `source_freeze_frame` constructs the closed canonical
`FreezeFrame` variant.

## Timeline Kernel

Every item separates its sequence placement from its source sampling:

- `record_range` determines where it appears.
- the closed source union determines what it contains.
- an optional mapping determines source time.
- typed visual/audio/text properties determine presentation.

Only media and sequence sources accept mappings. Source maps are linear or segmented curves; freeze is a
distinct source constructor. Curve interpolation is only
`linear` or `hold`. Linear and curve use `strict`, `hold-first`, `hold-last`, or
`hold-both`; freeze has no outside policy.

All record times become exact integer ticks. Validation rejects gaps, invalid
key order, illegal endpoint coverage, overflow, and mismatched durations.

## Composition Kernel

Composition is explicit:

- Layers establish deterministic stacking and timing.
- Schema-v9 transitions are centered-only true overlaps between adjacent visual items.
- Their exact record intersection is the transition duration; both real endpoint streams
  cover the full window, so codegen never completes an endpoint with held-frame `tpad`.
- Disabled items still belong to canonical topology, so a disabled third item cannot cross
  the overlap. Duration is an authored assertion; edits atomically update it with both ranges.
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

Resource constructors are closed to video, audio, image, font, LUT1D, and LUT3D.
Locations are `local` paths or pinned `remote` URIs. Video and audio selection
are independently `auto`, `disabled`, or an explicit index and lower to canonical stream intent.
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

Artifact values are typed recipes, not anonymous settings. Closed constructors separately own container,
codec, source, frame, canvas, numbering, analysis, and package choices. Lowering
creates one schema-v10 `RenderConfig` and one canonical `Deliverable` per artifact;
the canonical project requires minimum reader 10.
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
- Lowering emits only canonical schema version 10 constructs.
- Planning consumes canonical IR, never executable Surface syntax.
- Backend artifacts contain no unresolved source-language choices.
