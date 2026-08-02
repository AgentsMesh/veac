# Agent Authoring Guide

This guide is for agents that create or revise VEAC projects. Treat the complete
`.veac` source graph as typed source code: reuse static declarations, choose a
closed primitive, put it under the correct owner, compile, and validate.

## Three Artifacts, Three Jobs

An entry `.veac` plus imported modules is the authoring source of truth.
`SourceEditBatch` atomically changes typed expression sites in that graph.
Canonical JSON `EditBatch` atomically changes a canonical IR revision.

Do not mix the boundaries. `veac source-edit` recompiles source; `veac edit`
never parses or rewrites `.veac`. VEAC does not decompile edited IR to source.

## Ownership First

Keep this tree in working memory:

```text
project
|- settings, resources, entry, annotations
|- multicam groups
|- sequences
|  |- layers
|  |  `- items -> one source
|  |- transitions and relations
|  |- scoped Apply pipelines
`- deliveries -> typed artifacts
```

`multicam`, `delivery`, and `annotation` are project members. Layers, items,
relations, and Apply records belong to one sequence. Lowering maps layer/item
to canonical IR Track/Clip and hoists relations to the project collection.

## Minimal Authoring Source

```veac
project sample {
    settings {
        timebase 1/1000;
        canvas 1920px by 1080px;
        frame-rate 30fps;
        sample-rate 48000hz;
    }

    entry sequence main;

    sequence main {
        layer visual picture {
            item host-shot {
                record { at 0s; duration 5s; }
                source generated solid { color #18202aff; }
            }
        }
    }

    delivery preview {
        sequence main;
        raster { canvas 1920px by 1080px; frame-rate 30fps; captions discard; }
        artifact video preview {
            target file "preview.mp4";
            mux mp4 {
                layout standard;
                video h264 {
                    pixel-format yuv420p;
                    alpha opaque;
                    color-space source;
                    rate-control crf { value 23; }
                    gop automatic;
                    b-frames automatic;
                    profile automatic;
                    level automatic;
                }
                audio none;
                passes single;
                accelerator auto;
            }
        }
    }
}
```

Do not add a document version declaration.

## Reuse Before Copying

Use the focused [agent reuse guide](agent-reuse.md) for modules, presets, component instances,
nested composition, explicit parameter forwarding, and slot forwarding. The complete language
contract remains in [Compile-Time Programming](../language-reference/programming.md).

## Choose Closed Variants

Item sources are media, text, caption, generated, nested sequence, or multicam.
Use `source generated solid`, never `source color`. Subtitle files are delivery
artifacts. Media stream intent is `auto` or `disabled`; never invent indices.

## Express Time Explicitly

Record time answers where the item appears. Source time answers what portion of
the source is sampled. Use one mapping primitive:

```veac
mapping linear { from 4s; to 12s; outside strict; }
mapping freeze { source 7s; }
mapping curve {
    key start { at 0s; source 4s; interpolation linear; } key end { at 5s; source 12s; interpolation hold; }
    outside hold-last;
}
```

Curve keys use item-local record time and `linear` or `hold`; only media and sequence sources carry mappings.

## Build Ordered Processing

Use `pipeline` for ordered color/effect stages; do not flatten them. Apply
records target `scope composite-band`, `scope layer`, or `scope items` and keep
mix, matte, opacity, and blend typed.
LUT1D interpolation is `nearest`, `linear`, `cosine`, `cubic`, or `spline`;
LUT3D is `nearest`, `trilinear`, `tetrahedral`, `pyramid`, or `prism`.

## Route Audio And Deliver Captions

An audio layer may route to a bus; there is no independent `bus` declaration.
Audio stems select one master, track, or bus. Caption sidecars preserve captions
as data. See the [delivery reference](../language-reference/outputs.md).

## Deterministic Workflow

```bash
veac fmt project.veac --check
veac compile project.veac --emit-ir build/project.veac.json
veac check-ir build/project.veac.json
```

When source remains authoritative, inventory it and preview a `SourceEditBatch` before committing:

```bash
veac source-revision project.veac
veac source-index project.veac
veac source-edit project.veac source-edit.json --dry-run
veac source-edit project.veac source-edit.json
```

See the [source editing contract](../language-reference/source-editing.md).

For an IR edit, generate canonical JSON, not invented syntax:

```json,canonical-edit-batch
{
  "operation_id": "op_disable_host",
  "base_revision": 0,
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

Apply it atomically:

```bash
veac edit build/project.veac.json edits.json --output build/project-next.veac.json
veac check-ir build/project-next.veac.json
```

Use `--dry-run` to validate a batch without committing output. Preconditions,
locks, reference rewrites, and full-project validation are part of the edit
transaction. Never treat edited IR as a source-file patch.

## Agent Checklist

- Use only documented closed variants and typed units.
- Put every declaration under its implemented owner.
- Keep stable IDs and explicit references.
- Preserve ordering of layers, items, stages, processors, and edit operations.
- Preserve imports, comments, and untouched source bytes during targeted edits.
- Compile before planning or rendering.
- Reject unknown fields instead of smuggling property bags through strings.
- Validate after every generated source, SourceEditBatch, or EditBatch change.
