# CLI Reference

The CLI exposes one pipeline: a `.veac` source graph to canonical IR, plan, bundle, and artifacts.

## Source Commands

```bash
veac check <source.veac> [--diagnostic-format json]
veac fmt <source.veac> [--check | --stdout]
veac compile <source.veac> --emit-ir <project.json> [--revision N]
veac source-revision <source.veac>
veac source-index <source.veac>
veac source-edit <source.veac> <source-edit-batch.json> [--output <source.veac>] [--dry-run]
```

`check` resolves the complete source graph, evaluates pure expressions, expands static presets and
components, parses the core document, lowers it, and canonical-validates without media I/O.
`compile` runs the same frontend and writes deterministic canonical JSON. A programming source is
kept byte faithful by `fmt`; core single-file source uses semantic canonical formatting.

`source-revision` prints the exact graph SHA-256. `source-index` prints the same revision together
with a deterministic inventory of module-qualified source targets and their editable expressions.
`source-edit` applies a revisioned, preconditioned authoring edit and recompiles the graph before
atomic commit. It never reconstructs source from IR.
Dry runs and independent output copies do not create a source lock; in-place edits use the reserved
`.veac-source.lock` protocol and descriptor-relative staged replacement.
See [Source-Of-Truth Editing](language-reference/source-editing.md).

## Canonical Commands

```bash
veac check-ir <project.json>
veac edit <project.json> <edit-batch.json> [--output <project.json>] [--dry-run]
veac template inventory <project.json> [-o <inventory.json>]
veac template propose <project.json> <fill-request.json> -o <edit-batch.json>
```

Canonical edits are revision-aware, preconditioned, atomic, and followed by full validation.
`EditBatch` and `SourceEditBatch` are separate contracts for separate sources of truth.

## Artifact and Plan Commands

```bash
veac probe <media>
veac plan <project.json> [--bindings <bindings.json>] [--config <id>]
veac manifest <project.json> [--bindings <bindings.json>] [-o <manifest.json>]
veac package <project.json> --destination <dir> [--config <id>]
```

Probe snapshots bind media identity and stream metadata. Plan consumes canonical intent plus verified
bindings and prints the resolved plan. Manifest and package capture reproducible dependencies.

## Render Commands

```bash
veac render <project.json> [--bindings <bindings.json>] [--config <id>] [--destination <dir>]
```

`render` performs resolution, code generation, and guarded execution. Checkpoint-compatible completed
tasks are reused. Output locks prevent two processes from writing the same destination concurrently.

## Public Schemas

```bash
veac schema --contract project
veac schema --contract edit-batch
veac schema --contract source-edit-batch
veac schema --contract source-index
```

Schema output is strict JSON Schema. Unknown fields and variants are rejected by the corresponding
decoder; prose examples do not extend the contract.

## Diagnostics

Human diagnostics use a stable shape:

```text
error[AUTHORING_REFERENCE_NOT_FOUND]: typed reference target does not exist
  --> main.veac:18:23
  help: correct the reported condition and retry
```

JSON diagnostics include code, message, source span or JSON pointer, object identity where available, and suggested repair. Exit status is nonzero if any error is emitted.

## Examples and Tests

```bash
make check-examples
make build-examples
make serve-examples
make e2e
```

`build-examples` writes only under the ignored `examples-preview/` directory. It uses cataloged sources and the real CLI pipeline.

Run `veac <command> --help` for the exact installed flag set.
