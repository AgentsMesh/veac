# RFC: Executable Agent Source And Canonical IR

Status: Accepted and implemented

## Decision

VEAC has one executable, agent-oriented source language and one canonical JSON execution IR. Historical
source compatibility is out of scope.

```text
.veac source graph
  -> resolve + type/effect/stage verification
  -> verified Core v10
  -> bounded graph execution + Temporal residualization
  -> canonical JSON IR schema 9
  -> plan -> typed backend bundle -> artifacts
```

The source graph is the source of truth. JSON IR is a validation, interchange, caching and execution ABI;
it is never decompiled to recover `.veac`.

## Why A Language

A flat attribute surface is JSON with different punctuation. It cannot make ownership, typed handles,
source/record time, reusable construction, legal combinations or effects obvious to an agent. Open maps
also defer useful diagnostics until source spans and domain intent have disappeared.

VEAC instead supplies modules, immutable nominal values, pure functions, methods, closures, exhaustive
matching, bounded collections and closed Domain constructors. `main(Context) -> Project` is the only entry
ABI. Root `animate` declarations residualize approved Temporal leaves without making graph topology dynamic.

## Semantic Rules

1. Entity identity is an explicit typed constructor operand.
2. Ownership is formed by typed attachment methods, never property projection.
3. Cross-owner behavior is a first-class relation.
4. Values, variants and units are closed and typed.
5. Effects are ordered `Pure < LocalMutation < GraphEmit`.
6. Stages are ordered `Const < Build < Temporal`.
7. Topology is static; only approved scalar/vector leaf values may be Temporal.
8. Unknown calls, variants, fields and units fail before execution.
9. Runtime executes verified Core, not Surface syntax or arbitrary host callbacks.
10. Lowering is explicit and canonical validation follows every build.

## Reuse

Ordinary functions and methods are the component system. Nominal structs and enums carry reusable typed
configuration; modules publish stable APIs. Factories return Domain handles and attach children explicitly.
There is no parallel preset/component macro grammar, generated source interpolation or property bag.

Entity keys remain visible at call sites. Hygienic identity comes from typed owner paths and length-framed
canonical ID derivation, not hidden string concatenation.

## Canonical Boundary

Canonical IR uses exact rational time, stable typed IDs, deterministic ordering and closed serde schemas.
The current project envelope is schema version 9 with minimum reader version 9. IR contains graph facts and
residual Temporal programs, not modules, functions, closures, nominal declarations or Surface expressions.

This accepted media fragment is parsed by the IR documentation test so the RFC cannot drift from the
canonical ABI:

```json,veac-media
{
  "id": "med_logo",
  "identity": null,
  "kind": "image",
  "authorship": null,
  "probe": null,
  "source": {
    "type": "file",
    "uri": "assets/logo.png"
  },
  "stream_intent": {
    "audio": { "type": "disabled" },
    "video": { "type": "auto" }
  }
}
```

The authoring and canonical edit contracts are deliberately separate:

- `SourceEditBatch` addresses declarations and typed body sites in a revisioned `.veac` graph, then rebuilds
  and validates the complete program before commit.
- `EditBatch` atomically edits canonical IR for downstream tools. It never patches source.

## Time And Temporal Values

Record time, sequence time, item-local time, source time and delivery frame/sample time are distinct domains.
Source mappings transform item-local time into source time. An `animate` body may use typed implicit clocks
and Pure helpers; the compiler residualizes a verified canonical DAG bound to one approved sink.

The approved sink set is closed to visual position, scale, rotation, crop and opacity plus audio gain and
pan. A Temporal value cannot choose an entity, attach a child, change ordering or emit graph topology.

## Relations And Delivery

Transition, matte, sidechain, group and AV-link are typed relation facts. Schema-v9 transitions are
centered-only true overlaps: adjacent real visual streams must both cover the exact intersection, and the
backend may not manufacture endpoints with held-frame `tpad`.

A project-owned delivery selects a sequence and owns typed deliverables. Separate constructors model video,
image sequence, caption sidecar, audio stem, scope, audio file, animated image, still image and adaptive
package recipes. Lowering creates stable render-config and deliverable IDs; compatibility validates before
planning or execution.

## Consequences

- The legacy property frontend and preset/component expansion frontend are deleted.
- CLI, docs, examples and source editing all use the executable frontend.
- New mechanisms require a closed Domain operation, direct lowering, rejection coverage and proportional
  planning/codegen/E2E evidence.
- Compiler/runtime budgets are deterministic and cannot be reset by modules, calls or collection loops.
- Controlled implementation, test, documentation and tooling files remain below 200 lines.
