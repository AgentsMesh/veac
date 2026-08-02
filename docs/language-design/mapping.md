# Authoring To Canonical IR Mapping

This defines the semantic boundary between a `.veac` source graph, canonical
JSON IR schema version 5, and both transactions, not a property-name table.

## One Compilation Path

```text
entry .veac + imported modules
  -> resolve exports and pure expressions
  -> expand typed presets and sequence components
  -> core lexer/parser
  -> typed authoring Document
  -> reference and material lowering
  -> canonical project JSON (schema version 5, minimum reader 5)
  -> planner -> backend
```

`veac compile --emit-ir` runs the complete graph pipeline. A core-only file is
the degenerate one-node graph. Planner and backend code consume only fully
expanded canonical IR. Unknown symbols, syntax, types, variants, references, or
unsupported combinations fail before planning.

## Ownership Mapping

| Authoring source | Canonical IR schema 5 |
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
| project `delivery` | `Project.render_configs[]` |
| delivery `artifact` | one typed `Deliverable` |

Layer/item are authoring names; Track/Clip are canonical names. Delivery and
multicam declarations never become sequence children. IDs are lowered through
typed namespaces, so resources, sequences, tracks, clips, buses, render configs,
and deliverables are not interchangeable strings.

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
remain ordered. An audio-stem artifact selects exactly one source: master, one
track, or one routed bus.

A delivery lowers to one `RenderConfig`; each artifact lowers to one typed
`Deliverable` with a file, image-sequence pattern, or package target. The closed
kind union is video, image sequence, caption sidecar, audio stem, scope, audio
file, animated image, still image, or adaptive package. Typed recipe primitives
become codec/container/image/package enums and settings, never an untyped map.
No delivery is owned by a sequence; it holds a typed sequence reference.

## Separate Edit Boundaries

There is no inline `edit project { ... }` syntax. IR edits are canonical JSON:

```json,canonical-edit-batch
{
  "operation_id": "op_disable_host",
  "base_revision": 12,
  "atomic": true,
  "preconditions": [],
  "operations": [
    {
      "type": "set_clip_enabled",
      "clip_id": "itm_host_shot",
      "enabled": false
    }
  ]
}
```

Apply it to canonical project JSON:

```bash
veac edit project.veac.json edits.json --output project-next.veac.json
```

The editor checks revision/preconditions and locks, applies operations in order,
rewrites typed references where the operation contract requires it, validates
the complete project, and commits or rolls back the full batch. `--dry-run`
performs the same checks without writing output. It never patches `.veac` text.

When `.veac` is authoritative, `SourceEditBatch` targets a module-qualified
source node and expression site under an exact graph revision. The transaction
preserves untouched bytes and recompiles through canonical validation. It does
not reverse-map an IR edit. See [source editing](../language-reference/source-editing.md).

## Verification Invariants

- `parse(fmt(parse(source)))` is semantically stable.
- Canonical project JSON validates against schema version 5 with minimum reader 5.
- Source, artifact, target, and recipe unions are closed.
- Ownership and typed reference namespaces are preserved.
- Ordered collections remain ordered where order is semantic.
- Static expansion is deterministic, hygienic, bounded, and absent from IR.
- A source edit produces a valid recompiled graph or changes no source bytes.
- Editing a canonical project yields another fully valid canonical project.
