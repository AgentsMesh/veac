# CLI Reference

The CLI exposes one pipeline: current `.veac` authoring source to canonical IR, plan, bundle, and artifacts.

## Source Commands

```bash
veac check <source.veac> [--json]
veac fmt <source.veac> [--check]
veac compile <source.veac> --out <project.json> [--revision N]
```

`check` parses, semantically validates, lowers, and canonical-validates without probing or rendering. `fmt --check` exits nonzero when the source is not canonical formatted. `compile` writes deterministic canonical JSON.

## Canonical Commands

```bash
veac validate <project.json> [--json]
veac edit <project.json> --batch <edit.json> --out <project.json>
veac template inventory <project.json> --out <inventory.json>
veac template fill <project.json> --request <fill.json> --out <project.json>
```

Edits and template fills are revision-aware, preconditioned, atomic, and followed by full canonical validation.

## Artifact and Plan Commands

```bash
veac probe <project.json> --out <probe.json>
veac plan <project.json> [--probe <probe.json>] [--config <id>] --out <plan.json>
veac codegen <plan.json> --out <bundle.json>
```

Probe snapshots bind media identity and stream metadata. Plan consumes canonical intent plus probe facts. Codegen emits a typed backend bundle and performs no process execution.

## Render Commands

```bash
veac render <project.json> [--probe <probe.json>] [--config <id>] --out-dir <dir>
veac execute <bundle.json> --out-dir <dir>
veac resume <bundle.json> --out-dir <dir>
```

`render` performs probe, plan, codegen, and execution. `execute` runs an existing bundle. `resume` reuses only checkpoint-compatible completed tasks. Output locks prevent two processes from writing the same destination concurrently.

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
