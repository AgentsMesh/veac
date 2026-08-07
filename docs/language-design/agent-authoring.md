# Agent Authoring Guide

Treat the complete `.veac` source graph as typed executable code. Reuse declarations, choose a closed
constructor, attach values through their owner methods, then check and build. Canonical JSON is the
backend/interchange ABI, not a second authoring language.

## Three Artifacts

- `.veac` plus imported modules is the authoring source of truth.
- `SourceEditBatch` atomically changes typed sites in that source graph and rebuilds it.
- Canonical `EditBatch` atomically changes an explicit canonical IR revision.

Do not mix boundaries. `veac source-edit` never decompiles IR; `veac edit` never parses or rewrites source.

## Ownership

```text
Project
|- Resource, MulticamGroup, Annotation
|- Sequence -> Layer -> Item -> one Source
|          -> Relation and Apply
`- Delivery -> typed Deliverable
```

Construct a value, keep its typed handle, and attach it once to the correct owner. Never invent a flattened
canonical ID or smuggle an unknown field through metadata.

## Minimal Source

```veac
animate visual-opacity on clip(@sample, @main, @picture, @host-shot) {
  clamp(progress * 2.0, 0.0, 1.0)
}

fn main(context: Context) -> Project {
  let host = item(
    identifier("host-shot"), item_enabled(), during(0s, 5s),
    source_generated(generator_solid(#18202aff)), source_timing_native()
  );
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let picture = visual_layer(
    identifier("picture"), 0, placement_free(), state, track_routing_default()
  ).with_item(host);
  let timeline = sequence(
    identifier("main"), "Agent 示例",
    sequence_settings(canvas(1920px, 1080px), frame_rate(30, 1), 48000)
  ).with_layer(picture);
  project(identifier("sample"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
```

The root file owns exactly one `main(Context) -> Project`. Modules export functions, structs, enums and
methods; imported `main` cannot satisfy the ABI. `animate` targets a complete logical Item path and one
closed dynamic property.

## Reuse

Use module functions and nominal configuration values instead of copying constructor trees. A static
component is a typed factory/method returning a Domain value. Collection `map`/`for` can build repeated
values under deterministic budgets. Entity keys remain explicit operands at every call site.

## Closed Operations

Run `veac language-spec` before generating unfamiliar code. Standard-library symbols are not arbitrary
keywords: each Domain operation publishes numeric ID, parameter types, result type, effect, stage and
runtime/lowering action. Unknown calls, wrong units and effect/stage violations fail before execution.

Keep record time, clip time and source time distinct. Use `during`, `source_timing_native`, or a typed
`source_mapping`; use root `animate` for random-access dynamic leaf expressions. Transition relations must
use centered true overlap between complete real streams.

## Source Workflow

```bash
veac check main.veac
veac source-revision main.veac
veac source-index main.veac
veac source-edit main.veac source-edit.json --dry-run
veac build main.veac --emit-ir project.json
veac check-ir project.json
```

Source edits are revision-bound and preserve untouched bytes. Function/method/temporal bodies and nominal
declarations use closed source sites. An accepted edit re-resolves, verifies, executes, residualizes, lowers
and validates the complete graph before atomic commit.

For an explicit IR workflow, use canonical JSON:

```json,canonical-edit-batch
{
  "operation_id": "op_disable_host",
  "base_revision": 0,
  "atomic": true,
  "preconditions": [],
  "operations": [
    {
      "type": "set_clip_enabled",
      "clip_id": "itm_host-shot",
      "enabled": false
    }
  ]
}
```

Apply with `veac edit project.json edits.json --output project-next.json`, then `veac check-ir`. Never treat
the edited IR as a source patch.

## Checklist

- Use only machine-published closed constructors and typed units.
- Preserve stable source keys and explicit owner handles.
- Preserve semantic order of layers, items, stages, processors and operations.
- Keep topology no later than Build; Temporal values flow only to approved leaves.
- Preserve imports, comments and untouched bytes during targeted source edits.
- Check, build and validate before planning or rendering.
- Reject property bags, reflection, generated-source reparsing and partial output.
