# Architecture

## Pipeline

```text
.veac
  -> veac-lang typed authoring AST
  -> veac-ir canonical ProjectEnvelope
  -> veac-plan ResolvedRenderPlan
  -> veac-codegen BackendBundle
  -> veac-runtime executor
  -> FFmpeg tasks and direct artifacts
```

Typed edit and template-fill requests enter at the canonical project boundary and produce a newly validated revision before planning.

## Crate Ownership

| Crate | Owns | Does not own |
| --- | --- | --- |
| `veac-lang` | lexer, parser, spans, semantic validation, formatter, lowering | probing, render policy, FFmpeg strings |
| `veac-ir` | canonical serde model, IDs, invariants, edit contracts | surface syntax, filesystem access |
| `veac-artifact` | probe snapshots, source clocks, artifact identity | timeline semantics |
| `veac-plan` | graph resolution, streams, time maps, output plan | authoring recovery, command execution |
| `veac-codegen` | typed backend bundle, filters, direct artifacts | project mutation |
| `veac-runtime` | task execution, locks, checkpoints, output verification | authoring interpretation |
| `veac-template` | inventory, typed bindings, atomic fill proposals | planner behavior |
| `veac-cli` | command orchestration and stable diagnostics | duplicate compiler logic |

## Authoring Boundary

The authoring AST is deliberately separate from canonical IR. It retains:

- byte spans for each declaration and value;
- omitted versus explicit values;
- stable declaration order where order is semantic;
- local typed enums and references;
- syntax-level ownership.

Lowering explicitly maps every variant to canonical values. It creates stable prefixed IDs and validates the resulting envelope. Unknown fields never survive as opaque maps.

## Canonical IR

Canonical JSON is the deterministic interchange and transaction boundary. It contains only executable or reviewable facts. It uses exact rational time, typed IDs, closed variants, deterministic ordering, and strict validation.

Relations are first-class canonical facts. If planner compatibility requires an embedded projection, one projector derives it and validation checks the relation/projection bijection.

## Planning

Planning is the only layer allowed to combine canonical timeline intent with probe snapshots and source clocks. It resolves:

- sequence dependency order and nesting;
- material inputs and selected streams;
- record/source time mappings;
- multicam switch programs;
- visual/audio/caption components;
- fonts and LUTs;
- render config and deliverable compatibility.

Plans are deterministic inputs to codegen and contain no parser recovery state.

## Codegen and Runtime

Codegen emits a typed bundle instead of a shell command. Tasks declare inputs, outputs, dependencies, arguments, and verification expectations. Direct writers handle caption sidecars and other non-FFmpeg artifacts.

Runtime executes the bundle with output locking and checkpoint identity. It never reinterprets `.veac` or mutates canonical intent to make a backend command succeed.

## Validation Layers

```text
authoring: syntax, fields, units, references, owner-specific rules
canonical: cross-object invariants and deterministic ordering
planner: artifact availability, clocks, streams, resolved graph
codegen: faithful backend subset and filter compatibility
runtime: process success, output existence, probe verification
```

Errors should appear at the earliest layer with enough information to be precise. Later layers retain guards for hand-authored canonical JSON and external edit clients.

## Testing

Unit tests cover closed variants and validators. Integration tests cover parse/format/lower/validate, edit/template transactions, plan/codegen bundles, and CLI diagnostics. E2E tests run real FFmpeg/ffprobe and verify output media/artifacts.

Active code is held above 95% line coverage, and controlled implementation/tooling files stay below 200 lines. Legacy frontends are deleted rather than hidden from coverage.
