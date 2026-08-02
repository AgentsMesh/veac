# Agent Authoring Guide

This guide is for agents that create or revise VEAC projects. Treat `.veac` as
typed source code: choose a closed primitive, put it under the correct owner,
format it, compile it, and validate the canonical IR.

## Two Inputs, Two Jobs

VEAC has two machine-facing inputs: `.veac` describes a complete project, while
canonical JSON `EditBatch` describes atomic changes to schema-v5 JSON IR.

There is no second textual edit language. `veac edit` does not parse `.veac` or
rewrite authoring source; it reads a canonical project JSON plus an EditBatch
JSON and writes a new canonical project JSON.

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
transitions, relations, and Apply records belong to one sequence. An item source
is not a reusable declaration by itself. Lowering maps layer/item to canonical
IR Track/Clip and hoists typed relations into the project relation collection.

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

Do not add a document version declaration. `veac fmt` is the authority for
canonical spelling and layout.

## Choose Closed Variants

Item sources are exactly media, text, caption, generated, nested sequence, or
multicam. Their canonical heads are `source media resource`, `source text`,
`source caption`, `source generated <kind>`, `source sequence sequence`, and
`source multicam multicam`.

Use `source generated solid`, never `source color`. Caption text is an item
source; subtitle files such as SRT, VTT, or ASS are `artifact
caption-sidecar` recipes inside a delivery, never media sources.

Media resources select streams with only `auto` or `disabled`. These become
canonical stream intent; probe normalization records exact selected streams for
planning. Do not invent stream indices in `.veac`.

## Express Time Explicitly

Record time answers where the item appears. Source time answers what portion of
the source is sampled. Use one mapping primitive:

```veac
mapping linear { from 4s; to 12s; outside strict; }
mapping freeze { source 7s; }
mapping curve {
    key start { at 0s; source 4s; interpolation linear; }
    key end   { at 5s; source 12s; interpolation hold; }
    outside hold-last;
}
```

Curve keys are in item-local record time and permit only `linear` or `hold`.
Outside policies are `strict`, `hold-first`, `hold-last`, and `hold-both`;
freeze has none. Only media and sequence sources carry mappings.

## Build Ordered Processing

Use `pipeline` for ordered color and effect stages. A named `lut-1d` accepts
`nearest`, `linear`, `cosine`, `cubic`, or `spline`; `lut-3d` accepts `nearest`,
`trilinear`, `tetrahedral`, `pyramid`, or `prism`. Do not flatten stages.

Use `scope composite-band`, `scope layer`, or `scope items` for first-class
Apply records. Scope selects the target; pipelines preserve operation order;
`mix`, matte, opacity, and blend remain typed.

## Route Audio And Deliver Captions

An audio layer can end with `route bus <id>;`. Referencing a bus ID establishes
the routing identity; there is no independent `bus` declaration. Use processors
in order, route layers to buses, then select one `master`, `track`, or `bus`
source from each `artifact audio-stem`.

Use `artifact caption-sidecar` with `source caption-tracks` and a closed `encode`
recipe. Sidecars preserve captions as data; burned-in video does not replace
them. See the [delivery reference](../language-reference/outputs.md) for every
artifact and unit-bearing recipe.

## Deterministic Workflow

For a newly authored project:

```bash
veac fmt --check project.veac
veac compile --emit-ir project.veac --out build/project.veac.json
veac validate build/project.veac.json
```

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
veac edit build/project.veac.json edits.json --out build/project-next.veac.json
veac validate build/project-next.veac.json
```

Use `--dry-run` to validate a batch without committing output. Preconditions,
locks, reference rewrites, and full-project validation are part of the edit
transaction. If authoring source remains the source of truth, make the same
semantic change in `.veac` and recompile instead of treating edited IR as a
source-file patch.

## Agent Checklist

- Use only documented closed variants and typed units.
- Put every declaration under its implemented owner.
- Keep stable IDs and explicit references.
- Preserve ordering of layers, items, stages, processors, and edit operations.
- Let `veac fmt` normalize source; do not hand-normalize canonical JSON.
- Compile before planning or rendering.
- Reject unknown fields instead of smuggling property bags through strings.
- Validate after every generated source or EditBatch change.
