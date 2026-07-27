# V3 Authoring To Canonical IR Mapping

This document defines the implemented boundary between `.veac`, canonical JSON
IR schema V3, and transactional edits. It is a semantic mapping, not a
property-name translation table.

## One Compilation Path

```text
.veac
  -> lexer/parser
  -> typed authoring AST
  -> reference and material lowering
  -> canonical project JSON (schema version 3)
  -> planner -> backend
```

`veac fmt` operates on the AST. `veac compile --emit-ir` performs lowering.
Planner and backend code consume only canonical IR. Unknown syntax, variants,
fields, units, references, or unsupported combinations fail before planning.

## Ownership Mapping

| Authoring V3 | Canonical IR V3 |
| --- | --- |
| `project` | `Project` |
| `settings` | `ProjectSettings` |
| `resource` | typed `Material` plus policies/selections |
| project `multicam` | `Project.multicam_groups[]` |
| project `sequence` | `Project.sequences[]` |
| sequence `layer` | `Sequence.tracks[]` |
| layer `item` | `Track.clips[]` |
| sequence `transition` | `Sequence.transitions[]` |
| sequence `apply` / scope | `Sequence.applies[]` |
| sequence `relation` | typed entry in `Project.relations[]` |
| project `annotation` | `Project.annotations[]` |
| project `output` | `Project.render_configs[]` plus deliverable |

Layer/item are authoring names; Track/Clip are canonical names. Output and
multicam declarations never become sequence children. IDs are lowered through
typed namespaces, so a resource, sequence, track, clip, bus, and output ID are
not interchangeable strings.

## Resource And Stream Mapping

```veac
resource video host {
    locator local { path "media/host.mov"; }
    streams { video auto; audio disabled; }
    probe required;
}
```

This lowers to a video `Material` with a local locator, probe policy, video
intent `auto`, and disabled audio. Authoring permits only `auto` and `disabled`;
probe normalization records exact canonical stream selection and media facts
separately. Backend code does not repeat the selection heuristic.

Resource kinds are closed: `video`, `audio`, `image`, `font`, `lut-1d`, and
`lut-3d`. Locators are local paths or remote URIs with required SHA-256 identity;
their payloads stay typed. LUT and font files are materials, not stage strings.

## Source Mapping

The authoring source union maps as follows:

| `.veac` source | Canonical `ClipSource` |
| --- | --- |
| `media resource <id>` | `Media { material_id }` |
| `text { ... }` | typed text content |
| `caption { ... }` | typed text content with caption metadata |
| `generated <kind> { ... }` | `Generated { generator }` |
| `sequence sequence <id>` | `Sequence { sequence_id }` |
| `multicam multicam <id> { ... }` | `Multicam { group_id, switches }` |

Generated kinds are `transparent`, `silence`, `solid`, `gradient`, and `shape`.
The canonical union additionally has `FreezeFrame`, produced by lowering a
media source with `mapping freeze`; it is not a seventh authoring source syntax.

## Record And Source Time

Authoring record time lowers directly to integer-tick `Clip.record_range` under
the project timebase. Mapping lowering is variant-specific:

| Authoring mapping | Canonical result |
| --- | --- |
| `linear { from; to; outside; }` | `SourceMapping::Linear` |
| `curve { key...; outside; }` | `SourceMapping::TimeMap` segments |
| `freeze { source; }` on media | `ClipSource::FreezeFrame`; no mapping |
| `freeze { source; }` on sequence | one hold time-map segment |

Only media and sequence sources accept mappings. Curve key `at` values are
item-local record time. Adjacent keys become exact
segments whose record durations must sum to the clip record duration. Only
`linear` and `hold` are valid source-time interpolation modes. Linear and curve
carry `strict`, `hold-first`, `hold-last`, or `hold-both`; freeze carries none.

Validation rejects non-monotonic keys, illegal endpoint coverage, unsupported
easing, arithmetic overflow, gaps, and source/record duration inconsistency.

## Processing Mapping

An authoring `pipeline` lowers in source order to typed color/effect stages.
There is no generic option map and no raw backend-filter escape hatch.

```veac
lut show {
    resource show-look;
    interpolation tetrahedral;
}
```

The LUT reference resolves to a LUT material. LUT1D accepts `nearest`, `linear`,
`cosine`, `cubic`, or `spline`; LUT3D accepts `nearest`, `trilinear`,
`tetrahedral`, `pyramid`, or `prism`. Other interpolation names are parsed only
where their own primitive allows them and cannot leak into other semantics.

Scoped processing lowers to first-class Apply:

| Scope | Canonical target |
| --- | --- |
| `scope composite-band` | `ApplyTarget::CompositeBand` |
| `scope layer` | `ApplyTarget::Layer` |
| `scope items` | exact `ApplyTarget::ItemSet` |

Stage order, active ranges, mix, blend, opacity, mask, and matte references stay
typed through planning.

## Multicam Mapping

A project multicam group lowers its material-backed angles, sync method/master,
and tolerance to `Project.multicam_groups`. An item source references the group
and adds item-local record-time switches. Lowering resolves all angle and
material references; validation enforces non-empty, ordered, in-bounds switch
intervals.

## Audio And Delivery Mapping

`route bus mix;` on an audio layer lowers to `TrackRouting::AudioBus` with a
typed bus ID. There is no standalone bus declaration. Ordered audio processors
remain ordered. An audio-stem output selects exactly one source: master, one
track, or one routed bus.

Outputs are project members and form a closed union:

- video
- image sequence
- caption sidecar
- audio stem
- scope

Media outputs lower to a render config and typed deliverable. Caption sidecars
select a source sequence, `srt`/`vtt`/`ass`, and caption track IDs. Scope output
selects a sequence, scope kind/time/dimensions, and image format. No output is
owned by a sequence.

## Canonical EditBatch Only

There is no syntax such as `edit project { ... }`. IR edits are canonical JSON:

```json
{
  "operation_id": "op_disable_host",
  "base_revision": 12,
  "atomic": true,
  "preconditions": [],
  "operations": [
    {
      "type": "set_clip_enabled",
      "clip_id": "clip_host_shot",
      "enabled": false
    }
  ]
}
```

Apply it to canonical project JSON:

```bash
veac edit project.veac.json edits.json --out project-next.veac.json
```

The editor checks revision/preconditions and locks, applies operations in order,
rewrites typed references where the operation contract requires it, validates
the complete project, and commits or rolls back the full batch. `--dry-run`
performs the same checks without writing output. It never patches `.veac` text.

## Verification Invariants

- `parse(fmt(parse(source)))` is semantically stable.
- Canonical project JSON validates against schema V3.
- Source and output unions are closed.
- Ownership and typed reference namespaces are preserved.
- Ordered collections remain ordered where order is semantic.
- Editing a canonical project yields another fully valid canonical project.
