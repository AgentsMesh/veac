# CLI Reference

The CLI exposes one executable VEAC frontend. Every source command consumes the same parsed,
typed, verified program model before canonical JSON IR, planning, bundling, and rendering.

## Source Commands

```bash
veac check <source.veac> [--inputs <manifest.json>] [--input NAME=VALUE]... \
  [--package-root <package-root>]...
veac fmt <source.veac> [--check | --stdout] [--package-root <package-root>]...
veac build <source.veac> --emit-ir <project.json> [--inputs <manifest.json>] \
  [--input NAME=VALUE]... [--material-root <dir>] [--package-root <package-root>]... [--revision N]
veac source-revision <source.veac> [--package-root <package-root>]...
veac source-index <source.veac> [--package-root <package-root>]...
veac source-edit <source.veac> <source-edit-batch.json> \
  [--inputs <manifest.json>] [--input NAME=VALUE]... [--package-root <package-root>]... \
  [--output <source.veac>] [--dry-run]
veac language-spec [--schema]
veac package api <package-root>
veac package inspect <package-root>
veac package search <local-store> <query>
```

含 `input` 声明的 source 必须向 `build`、`check` 和 `source-edit` 显式绑定完整值。
`--inputs <build-inputs.json>` 提供 versioned manifest；可重复的 `--input NAME=VALUE` 提供适合脚本和
locale matrix 的轻量绑定。两者可同时使用：manifest 是 base，同名 inline binding 显式覆盖它。
CLI 总是先 prepare source，再按声明的精确类型解码 inline value，不根据 value 外形猜类型；text 按第一个
`=` 分隔并保留后续 `=`。重复 inline name、未知或缺失 name、错误类型、单位和 enum variant 都会在 graph
发布前失败。参数顺序、manifest/inline 来源和等价单位写法不改变最终 typed input digest。

manifest value 支持 primitive tag，以及按声明的 nominal type 解析的 payloadless enum tag
`{"type":"enum","value":"Variant"}`；inline enum 直接写 variant，例如 `--input locale=zh-Hans`。
JSON Schema 由 `veac schema --contract build-inputs` 输出。

`build` prepares verified Core, executes the root-local
`fn main(context: Context) -> Project` in one bounded transaction, lowers the frozen domain graph,
and validates deterministic canonical JSON. Legacy project-block authoring is not a CLI frontend.

Local material URIs are portable paths relative to a machine-local material root. `build` defaults
that root to the source root and requires IR containing local materials to stay there. An explicit
`--material-root` declares the future resolution base and permits publishing the IR into a detached
build directory; the path is never serialized into canonical IR. Stdout has no implied new base.

`check` performs the complete executable pipeline without media I/O and never retries through a
different frontend. Root entries must define `fn main(context: Context) -> Project`; legacy project
blocks fail closed. `fmt` parses and resolves either an entry or a standalone module before changing
layout, preserves every non-trivia token and comment in source order, then parses and resolves the
result again. It is deterministic, idempotent, and emits exactly one trailing newline. `--check`
returns `FORMAT_REQUIRED` instead of writing when layout differs; `--stdout` prints the formatted
source without writing; the default mode publishes one atomic in-place replacement.

`source-revision` prints `authored_source_graph_sha256` for editable project bytes and
`complete_source_graph_sha256` for the frozen project/package/ABI source graph, authority, and routes.
`source-index` v11 is static introspection: it needs no runtime bindings, embeds that same dual revision,
and publishes the resolved Build-input signature plus a deterministic inventory of editable project targets.
`source-edit` applies a revisioned, preconditioned edit and rebuilds the executable graph before
atomic commit. Every accepted edit re-runs verified Core, graph lowering, and canonical IR
validation. It never reconstructs source from IR.
All six source commands accept repeatable `--package-root` flags. Each root is a complete, exact,
locally verified package contract; no environment store, network, install hook, version solving, or
`latest` fallback is consulted. The supplied root list is the source-resolution contract and must be
repeated consistently across `build`, `check`, `fmt`, `source-revision`, `source-index`, and
`source-edit`. Mounted package bytes, authority, and routes contribute to the complete compilation
identity, while the index's editable modules exclude them. Source-edit v9 can diagnose a targeted
`packages/name@version/...` module as read-only and never writes a package root.
所有 source graph reader 会在 reserved `.veac-source.lock` 上持有 shared snapshot lock，in-place
writer 在完整 publish 期间持有 exclusive lock。因此多模块加载不会观察到不同 transaction generation。
read-only 命令、dry run 或独立 output 可以创建这个持久 coordination 文件，但不会修改 `.veac`；
in-place commit 继续使用 descriptor-relative staged replacement。
See [Source-Of-Truth Editing](language-reference/source-editing.md).
`language-spec` prints the syntax vocabulary, standard-library surface, and numeric domain opset
as canonical JSON; `--schema` prints its strict JSON Schema in the same canonical encoding. See
[Language Vocabulary](language-reference/vocabulary.md).

`package api` 验证 manifest、lock、完整源码闭包和 compiler-derived module interface，然后输出签入的
canonical API metadata。`package inspect` 输出 root 与精确依赖的身份、入口、content digest 和 API；
`package search` 只扫描显式给出的本地 store 一级目录，并按 exact package identity 排序。三个命令均不
读取环境变量、联网、执行 install hook 或选择隐式 `latest`。参见
[Package 合同](language-reference/packages.md)。

## 工程工作区命令

```bash
veac project check <project.veac> [--package-root <package-root>]...
veac project inspect <project.veac> [--package-root <package-root>]...
veac project graph <project.veac> [--package-root <package-root>]...
veac project build <project.veac> [--receipt <path>] [--package-root <package-root>]...
veac project evidence <project.veac> [--receipt <path>] [--package-root <package-root>]...
veac project test <project.veac> [--receipt <path>] [--package-root <package-root>]...
```

`check` 验证 authored manifest 和 resolved DAG；`inspect`、`graph` 输出 canonical 中间合同。
三个执行命令都运行完整 DAG、verified CAS 和 delivery。六个命令均只挂载重复
`--package-root` 显式给出的 exact package；manifest、target planning 与 backend execution 使用同一
集合，不读取环境 store 或隐式搜索目录。package root 不能与 source、material、build、cache 或
delivery authority 重叠。`evidence` 保留失败断言而不做 gate；`test` 在 bundle 和 receipt 发布后对
`fail`/`error` 返回非零状态。参见
[工程工作区](language-reference/project-workspaces.md)和[证据与验收](language-reference/evidence.md)。

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
veac probe <project.json> --material <material-id> [--material-root <dir>]
veac derive <media> <media-artifact-request.json> --store <artifact-store>
veac ingest-analysis <media> <analysis-ingestion-request.json> --store <artifact-store>
veac plan <project.json> [--material-root <dir> | --bindings <bindings.json>] [--config <id>]
veac manifest <project.json> [--material-root <dir> | --bindings <bindings.json>] [-o <manifest.json>]
veac bundle <project.json> --destination <dir> [--material-root <dir> | --bindings <bindings.json>] [--config <id>]
```

Probe snapshots bind media identity and stream metadata. With `--material`, `probe` strictly decodes
the canonical project, resolves the selected material's material-root-relative URI, verifies its authored
SHA-256 identity, and applies its typed video/audio stream intent. Without `--material`, it retains
the deterministic automatic-selection behavior for a direct media path. Plan consumes canonical
intent plus verified bindings and prints the resolved plan. `derive` executes only locally supported FFmpeg derivations.
`ingest-analysis` validates a closed typed external analysis result, its source identity, producer,
canonical digest, and cache identity before publication. Manifest and bundle capture reproducible
dependencies.

Canonical consumers default the material root to the project JSON directory. Pass `--material-root`
when IR and source assets live in separate trees. It is an execution-context alternative to
`--bindings`, so the two flags conflict. The root must be an existing directory; material resolution
follows paths only when their canonical target remains inside that root. A symlink used as the root
itself is allowed, while a material symlink escaping it fails with `MATERIAL_OUTSIDE_ROOT`.

## Render Commands

```bash
veac render <project.json> [--material-root <dir> | --bindings <bindings.json>] [--config <id>] [--destination <dir>]
```

`render` performs resolution, code generation, and guarded execution. Checkpoint-compatible completed
tasks are reused. Output locks prevent two processes from writing the same destination concurrently.
The default deliverable directory and `.veac-artifacts` remain anchored to the project JSON directory;
`--material-root` changes input resolution only.

## Public Schemas

```bash
veac schema --contract project
veac schema --contract project-manifest
veac schema --contract resolved-project-graph
veac schema --contract evidence-suite
veac schema --contract observation-plan
veac schema --contract evidence-report
veac schema --contract evidence-bundle
veac schema --contract evidence-provenance
veac schema --contract edit-batch
veac schema --contract source-edit-batch
veac schema --contract source-index
veac schema --contract language-spec
veac schema --contract analysis-ingestion-request
veac schema --contract analysis-result
```

Schema output is strict JSON Schema. Unknown fields and variants are rejected by the corresponding
decoder; prose examples do not extend the contract.

## Diagnostics

Human diagnostics use a stable shape:

```text
error[PROGRAM_TEMPORAL_TARGET]: typed animation target does not exist
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
