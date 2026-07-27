# VEAC Examples

Every directory contains one canonical authoring source at `examples/<id>/main.veac`. The gallery catalog is the source of truth for discovery, preview order, capability evidence, preview windows, and expected deliverables:

```text
examples/catalog/gallery.json
examples/catalog/mechanisms/*.json
examples/capabilities.json
```

The catalog and filesystem must contain exactly the same source set. There is no legacy example frontend.
`gallery.json.examples` owns one title, summary, and non-empty set of `{ cue, expect }`
checks for every source. A cue may identify media time, audio, IR, plan, or a rendered
artifact; presentation copy must not claim that an invisible mechanism is pixel evidence.

## Commands

```bash
make check-examples
make build-examples
make serve-examples
make clean-examples
```

`check-examples` performs all of the following for every cataloged source:

```text
parse -> format -> parse -> format-idempotence
      -> lower to canonical JSON IR
      -> canonical validation
```

It also validates capability metadata, gallery target ownership, workflow evidence, unique directory registration, and one `.veac` source per example directory.

`build-examples` compiles and renders every cataloged example into the ignored `examples-preview/` directory. It prepares generated fixture media and a deterministic preview font inside that directory, plans/renders with the real CLI and FFmpeg, verifies declared deliverables, and creates `examples-preview/index.html`.

The build never rewrites canonical relation facts or deletes embedded projections. For projects without a video deliverable, it injects a typed preview output into the generated `.veac` copy before compilation; the checked-in example remains unchanged.

## Coverage Map

- `minimal`, `hello-world`: project skeleton and basic generated/text sources.
- `timeline-source-time`, `speed-demo`: trim, speed, curve, and freeze mapping.
- `nested-and-multicam`: nested sequence and typed multicam switch program.
- `generated-graphics`: transparent, silence, solid, gradient, and shape sources.
- `transforms-and-animation`, `blend-modes`, `masks-and-mattes`: visual pipelines.
- `apply-scopes`: CompositeBand, Layer, exact ItemSet, ordered stage stack, and Apply mix mask.
- `transitions`, `transition-gallery`: canonical relation-based transitions.
- `audio-processing`, `executable-mechanisms`: processors, routing, group, and AV-link.
- `color-grade`, `advanced-color`: ordered color stages and LUT references.
- `text-overlay`, `text-layout`, `text-animation`: structured text value objects.
- `captions-and-sidecars`: caption source, speaker, style, and sidecar selection.
- `template-fill`: item-owned media and text template slots.
- `delivery-formats`: all five typed output variants.
- `all-features`, `agentsmesh-intro-15s`: cross-mechanism integration projects.

Capability rows should point to a precise gallery target and record window. A token occurring in source is not accepted as mechanism evidence; the integration test must be able to lower the source to the corresponding typed IR.
