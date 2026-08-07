# Architecture

## Pipeline

```text
.veac entry + confined imported modules
  -> executable Surface -> typed HIR -> verified Core v10
  -> bounded evaluator + one graph transaction -> freeze
  -> authored animate declarations -> residual Temporal programs
  -> direct ProjectEnvelope
  -> veac-ir canonical validation
  -> veac-plan ResolvedRenderPlan
  -> veac-codegen BackendBundle
  -> veac-runtime executor
  -> FFmpeg tasks and direct artifacts
```

An entry must provide one root-local `fn main(context: Context) -> Project`. Imported modules provide
reusable declarations but cannot provide the entry. Planner and backend crates never receive Surface
AST, Typed HIR, Core, graph handles, or compile-time declarations. The executable path produces a newly
validated canonical result before planning.

## Crate Ownership

| Crate | Owns | Does not own |
| --- | --- | --- |
| `veac-lang` | source graph, Typed HIR/Core v10, bounded evaluator, temporal residualization, graph transaction/freeze, direct lowering, source transactions | probing, render policy, FFmpeg strings |
| `veac-ir` | canonical serde model, IDs, invariants, edit contracts | surface syntax, filesystem access |
| `veac-artifact` | probe snapshots, source clocks, artifact identity | timeline semantics |
| `veac-plan` | graph resolution, streams, time maps, output plan | authoring recovery, command execution |
| `veac-codegen` | typed backend bundle, filters, direct artifacts | project mutation |
| `veac-runtime` | task execution, locks, checkpoints, output verification | authoring interpretation |
| `veac-template` | inventory, typed bindings, atomic fill proposals | planner behavior |
| `veac-cli` | command orchestration and stable diagnostics | duplicate compiler logic |

## Executable Programming Boundary

The executable entry owns exactly one root-local `main(Context) -> Project`; an imported `main`
cannot satisfy that ABI. Imported files own one anonymous `module {}` and export named functions,
nominal types, methods, and other supported declarations. Import resolution is rooted at the entry
directory, cycle checked, and protected against absolute paths, traversal, and symlink escape.

An imported file uses the anonymous `module {}` kind marker. Its loader-provided source ID is the
only module identity, and import aliases are lexical caller choices; no dead module self-name is
carried into resolution. Declarations, expression symbol segments, and source-edit target names
share one canonical ASCII name contract.

Functions, closures, structs, closed enums, exhaustive match, methods, collections, and control flow
resolve to Typed HIR and then numeric-ID Core v10. The verifier checks types, dominance, calls,
budgets, effect/stage metadata, and the pinned domain opset before execution. One bounded evaluator
owns one graph transaction shared by `main`, direct calls, methods, and closures. The host creates
`Context` inside that arena; opaque graph handles cannot cross graphs or escape ordinary results.

The effect lattice is `Pure | LocalMutation | GraphEmit`; the stage lattice is
`Const < Build < Temporal`. Aggregate shape and leaf stage are tracked separately. `map` and
Surface `for` may run bounded `GraphEmit` callbacks, while `filter` and `fold` callbacks remain
`Pure`. Entity identity, ownership, order, collection size, and source/resource selection must be at
most Build: **static topology, dynamic leaf values**.

Freeze requires one connected Project with a valid entry. The frozen graph lowers directly to
canonical `ProjectEnvelope`; executable production never renders `.veac`, reparses a generated
Document, or dispatches runtime operations by source string. Opset v7 has 214 closed Domain types and
581 numeric operations covering resources, timeline, transform, text/caption, audio, effects,
transitions, multicam, annotation, template and delivery mechanisms. Root `animate` declarations compile
through the same function system and residualize approved dynamic leaves into canonical programs.
The user-facing contract is [Executable Build](language-reference/executable-build.md).

## Source-Of-Truth Boundary

The exact `.veac` source graph remains the authoring source of truth. A graph revision hashes every
source byte. `SourceEditBatch` addresses typed source nodes and closed expression/body/declaration
sites, preserves untouched bytes, and must re-resolve, execute, lower, and validate under
an exclusive source-root advisory lock. Commit revalidates root, lock, parent, and target identities
plus every module byte before descriptor-relative staged replacement. VEAC never decompiles an
edited canonical project back into source.

## Canonical IR

Canonical JSON is the deterministic backend execution, interchange, and IR transaction boundary. It
contains only frozen executable or reviewable facts: no imports, Surface AST, graph handles,
presets, components, instances, scripts, or source-edit metadata. It uses exact rational time,
typed IDs, closed variants, deterministic ordering, and strict validation.

Local material URIs remain portable and relative; canonical JSON never records a host path. CLI
hydration supplies a canonical `material root` execution context, defaulting to the IR directory or
set explicitly with `--material-root`. This context is separate from the source-module root and from
is an alternative to machine-local execution bindings. Resolution rejects canonical targets outside the root, while render
outputs and the artifact store remain owned by the IR directory.

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

## Typed Artifact ABI

Artifact contract v2 has one source of truth for artifact type. `ArtifactParameters` is a closed,
tagged variant; `ArtifactDescriptor::kind()` is derived from that variant and is never serialized as
a second independently editable field. Parameters are typed structures, not JSON property bags.

Dependencies use the closed `ArtifactDependencyRole` enum. Provider outputs bind a validated nominal
slot into both their transport envelope and descriptor identity, so arbitrary role strings cannot
change routing without changing the artifact key. Unknown parameter kinds, fields, dependency roles,
unsorted dependencies, duplicate identities, and over-budget values fail closed.

FFmpeg derivation accepts only locally executable media specs. External analysis enters through a
versioned `AnalysisIngestionRequest` containing a closed typed descriptor/result envelope; the source,
producer, descriptor, and canonical result digest all participate in artifact identity. VEAC does not
advertise an analyzer that the runtime cannot execute.

## Codegen and Runtime

Codegen emits a typed bundle instead of a shell command. Tasks declare inputs, outputs, dependencies, arguments, and verification expectations. Direct writers handle caption sidecars and other non-FFmpeg artifacts.

Runtime executes the bundle with output locking and checkpoint identity. It never reinterprets `.veac` or mutates canonical intent to make a backend command succeed.

Artifact reuse binds the exact backend implementation, not only a manually maintained version. The
codegen identity hashes a controlled source inventory, workspace manifests and lockfile, Rust
toolchain, target, enabled features, package version, render/core/domain/temporal opsets, and every
FFmpeg plugin adapter descriptor and implementation ID. Runtime adds its own controlled source and
build identity. FFmpeg configuration and input policy complete the producer fingerprint used by
render segments and checkpoints; direct-write checkpoints bind the combined codegen/runtime
identity. Manual contract versions remain explicit audit fields, but are only one identity input.

## Validation Layers

```text
program: imports, visibility, types, call graph, effect/stage sinks, Core verification
build: deterministic budgets, graph affinity/ownership, transaction rollback, freeze
temporal: pure residual Core, clock ownership, closed sink typing, deterministic IDs
canonical: cross-object invariants and deterministic ordering
planner: artifact availability, clocks, streams, resolved graph
codegen: faithful backend subset and filter compatibility
runtime: process success, output existence, probe verification
```

Errors should appear at the earliest layer with enough information to be precise. Later layers retain guards for hand-authored canonical JSON and external edit clients.

## Testing

Unit tests cover closed variants, resolution, Core verification/runtime, graph rollback/freeze,
temporal residualization, and validators. Integration tests cover executable source to canonical IR,
source edits, plan/codegen bundles, and CLI diagnostics. E2E tests run real FFmpeg/ffprobe
only where media output is the behavior under test.

Active code is held above 95% line coverage, and controlled implementation/tooling files stay below 200 lines. Legacy frontends are deleted rather than hidden from coverage.
