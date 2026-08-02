# VEAC Language Reference

VEAC is an agent-oriented authoring language for deterministic video editing. It is not a textual JSON encoding. The source graph owns reusable declarations, intent, source spans, omitted state, and closed variants; canonical JSON is the execution IR.

The only supported compilation path is:

```text
entry .veac + modules
  -> resolve + pure evaluation + static expansion
  -> typed authoring Document
  -> canonical JSON IR
  -> plan -> FFmpeg/artifacts
```

The current canonical project envelope uses schema version 5 and minimum reader 5.

Core algebra:

```text
Project   = settings + resources + entry + multicams + sequences + annotations + deliveries
Delivery  = sequence + optional raster + typed artifacts
Sequence  = layers + relations + applies
Layer     = ordered items + optional audio routing
Item      = source + record span + optional source mapping + modifiers + optional template slot
Parameter = constant<T> | curve<T>
```

References are typed by their grammar position (`resource`, `sequence`, `layer`, `item`, `group`, `angle`, `track`, or `bus`). Unknown kinds, fields, duplicate fields, wrong units, and unresolved references are errors.

- [Project and resources](project.md)
- [Compile-time modules, expressions, presets, and components](programming.md)
- [Source-of-truth editing](source-editing.md)
- [Stable source addressing](source-addressing.md)
- [Timeline and mapping](timeline.md)
- [Sources](sources.md)
- [Modifiers and relations](modifiers-relations.md)
- [Text, captions, and audio](text-caption-audio.md)
- [Deliveries and artifacts](outputs.md)

Every public mechanism has an executable source under [`examples/`](../../examples/). `make check-examples` resolves each cataloged source graph, expands it, lowers it, and validates the resulting canonical project.
