# VEAC Language Reference

VEAC is an agent-oriented authoring language for deterministic video editing. It is not a textual JSON encoding. The surface language owns intent, source spans, omitted state, and closed variants; canonical JSON is the execution IR.

The only supported compilation path is:

```text
.veac -> typed authoring AST -> canonical JSON IR -> plan -> FFmpeg/artifacts
```

Core algebra:

```text
Project   = settings + resources + entry + multicams + sequences + annotations + outputs
Sequence  = layers + relations + applies
Layer     = ordered items + optional audio routing
Item      = source + record span + optional source mapping + modifiers + optional template slot
Parameter = constant<T> | curve<T>
```

References are typed by their grammar position (`resource`, `sequence`, `layer`, `item`, `group`, `angle`, `track`, or `bus`). Unknown kinds, fields, duplicate fields, wrong units, and unresolved references are errors.

- [Project and resources](project.md)
- [Timeline and mapping](timeline.md)
- [Sources](sources.md)
- [Modifiers and relations](modifiers-relations.md)
- [Text, captions, and audio](text-caption-audio.md)
- [Outputs](outputs.md)

Every public mechanism has an executable source under [`examples/`](../../examples/). `make check-examples` parses, formats, lowers, and validates every cataloged example.
