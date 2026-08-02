# Architecture

## Pipeline

```text
.veac entry + confined imported modules
  -> surface parse and source graph resolution
  -> exact pure expression evaluation
  -> typed preset/component hygienic expansion
  -> veac-lang core authoring Document
  -> veac-ir canonical ProjectEnvelope
  -> veac-plan ResolvedRenderPlan
  -> veac-codegen BackendBundle
  -> veac-runtime executor
  -> FFmpeg tasks and direct artifacts
```

Planner and backend crates never receive compile-time declarations. Source edits enter before
resolution and expansion; canonical edits and template fills enter at the canonical project
boundary. Both paths produce a newly validated result before planning.

## Crate Ownership

| Crate | Owns | Does not own |
| --- | --- | --- |
| `veac-lang` | source graph, modules, pure expressions, static expansion, core parser, spans, lowering, source transactions | probing, render policy, FFmpeg strings |
| `veac-ir` | canonical serde model, IDs, invariants, edit contracts | surface syntax, filesystem access |
| `veac-artifact` | probe snapshots, source clocks, artifact identity | timeline semantics |
| `veac-plan` | graph resolution, streams, time maps, output plan | authoring recovery, command execution |
| `veac-codegen` | typed backend bundle, filters, direct artifacts | project mutation |
| `veac-runtime` | task execution, locks, checkpoints, output verification | authoring interpretation |
| `veac-template` | inventory, typed bindings, atomic fill proposals | planner behavior |
| `veac-cli` | command orchestration and stable diagnostics | duplicate compiler logic |

## Programming Boundary

The entry file owns one project plus compile-time declarations. Imported files own one module and
export only named constants, typed presets, and sequence components. Import resolution is rooted at
the entry directory, cycle checked, and protected against absolute paths, traversal, and symlink
escape.

An imported file uses the anonymous `module {}` kind marker. Its loader-provided source ID is the
only module identity, and import aliases are lexical caller choices; no dead module self-name is
carried into resolution. Declarations, expression symbol segments, and source-edit target names
share one canonical ASCII name contract.

Constants and component arguments are typed pure expressions. Presets compose only at a matching
closed semantic site. Components have typed parameters and source slots; each instance expands into
a sequence, and local `@id` names receive reserved, length-framed generated IDs. Expansion is
deterministic, injective, collision-checked, bounded, and records provenance back to the source
graph.

## Authoring Boundary

The fully expanded authoring `Document` is deliberately separate from canonical IR. It retains:

- byte spans for each declaration and value;
- omitted versus explicit values;
- stable declaration order where order is semantic;
- local typed enums and references;
- syntax-level ownership.

Lowering explicitly maps every variant to canonical values. It creates stable prefixed IDs and validates the resulting envelope. Unknown fields never survive as opaque maps. Compile-time symbols
are already gone at this boundary.

The exact `.veac` source graph remains the authoring source of truth. A graph revision hashes every
source byte. `SourceEditBatch` addresses typed source nodes and closed expression sites, preserves
untouched bytes, and must re-resolve, expand, lower, and validate under an exclusive source-root
advisory lock. Commit revalidates root, lock, parent, and target identities plus every module byte
before descriptor-relative staged replacement. VEAC never
decompiles an edited canonical project back into source.

## Canonical IR

Canonical JSON is the deterministic execution, interchange, and IR transaction boundary. It
contains only fully expanded executable or reviewable facts: no imports, constants, expressions,
presets, components, instances, scripts, or source-edit metadata. It uses exact rational time,
typed IDs, closed variants, deterministic ordering, and strict validation.

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
program: imports, visibility, expression types, preset/component expansion
authoring: core syntax, fields, units, references, owner-specific rules
canonical: cross-object invariants and deterministic ordering
planner: artifact availability, clocks, streams, resolved graph
codegen: faithful backend subset and filter compatibility
runtime: process success, output existence, probe verification
```

Errors should appear at the earliest layer with enough information to be precise. Later layers retain guards for hand-authored canonical JSON and external edit clients.

## Testing

Unit tests cover closed variants, resolution, expansion, and validators. Integration tests cover
source graph to canonical IR, both edit boundaries, plan/codegen bundles, and CLI diagnostics. E2E
tests run real FFmpeg/ffprobe and verify output media/artifacts.

Active code is held above 95% line coverage, and controlled implementation/tooling files stay below 200 lines. Legacy frontends are deleted rather than hidden from coverage.
