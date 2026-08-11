# Getting Started

## Prerequisites

- Rust toolchain `1.85.0`
- FFmpeg 8.0 and ffprobe for rendering and E2E tests
- jq for example catalog/build tooling

```bash
make doctor
make build
make install
```

## Write an Executable Project

Create `main.veac`:

```veac
animate visual-opacity on clip(@intro, @main, @content, @background) {
  clamp(progress * 2.0, 0.0, 1.0)
}

fn main(context: Context) -> Project {
  let background = item(
    identifier("background"), item_enabled(), during(0s, 4s),
    source_generated(generator_solid(#18202aff)), source_timing_native()
  );
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let content = visual_layer(
    identifier("content"), 0, placement_free(), state, track_routing_default()
  ).with_item(background);
  let timeline = sequence(
    identifier("main"), "入门时间线",
    sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000)
  ).with_layer(content);
  project(identifier("intro"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
```

`main(Context) -> Project` 是视频程序入口 ABI。`animate` 在闭合属性和稳定 logical path 上声明
typed animation；body 是 verified Core，不是 JSON expression 或 property bag。多目标工程使用独立的
`workspace() -> ProjectManifest` host entry，验收合同使用 `evidence() -> EvidenceSuite`。

## Check, Build, and Inspect

```bash
veac check main.veac
veac fmt --check main.veac
veac build main.veac --emit-ir project.json
veac check-ir project.json
veac plan project.json > plan.json
```

For a complete project whose local assets live under `project/`, keep generated IR outside the
authored tree by declaring the same material root at publication and consumption:

```bash
mkdir -p build/runtime
veac build project/main.veac --material-root project --emit-ir build/runtime/project.json
veac plan build/runtime/project.json --material-root project > build/runtime/plan.json
veac render build/runtime/project.json --material-root project
```

`--material-root` is machine-local execution context and is not written into IR or plan hashes. It
cannot be combined with `--bindings`. Without it, canonical commands resolve local material URIs from
the project JSON directory, preserving the self-contained project-tree default.

`veac fmt --check` 仅检查规范布局；需要改动时返回 `FORMAT_REQUIRED`。使用 `veac fmt main.veac`
原子写回，或使用 `--stdout` 预览而不修改源码。格式化会保留 token spelling 与注释，并在改写前后
重新完成语法和语义验证。

Diagnostics include a stable code and exact source span. Unknown symbols, wrong units, invalid owners,
effect/stage violations, duplicate animation sinks and incomplete graph topology fail before IR is
published. Build emits canonical project envelope schema v10 and performs no media probing.

The small source above intentionally has no delivery. To render a complete checked-in project
without publishing generated files into its source tree:

```bash
make build-examples EXAMPLES=minimal
```

The `minimal` example owns a typed video delivery. Centered transitions require true overlap between
two complete real stream ranges; the backend never creates held-frame endpoint padding. Its generated
IR, render plan, logs and deliverables remain under the ignored `examples-preview/minimal/` directory.

## 构建多目标工程

当一个项目包含多语言、多个 profile、素材派生和自动验收时，由 `project.veac` 统一声明 source、
material、build、cache、delivery 根目录以及 target DAG：

```bash
veac project check project.veac
veac project graph project.veac
veac project build project.veac --receipt build/receipt.json
veac project evidence project.veac --receipt build/evidence.json
veac project test project.veac --receipt build/test.json
```

完整合同见[工程工作区](language-reference/project-workspaces.md)和[证据与验收](language-reference/evidence.md)。
`.veac` 是真源；上述命令生成的 canonical JSON、CAS payload、receipt 和交付目录都是可重建产物。

## Preview Every Mechanism

```bash
make build-examples
make serve-examples
```

The ignored `examples-preview/` directory contains canonical projects, resolved plans, render logs,
deliverables and a Chinese gallery entry for every cataloged example. Normal CI renders only six
representative smoke examples; full gallery FFmpeg rendering remains a local or manual/weekly workflow.

## Run Guards

```bash
make fmt-check
make structure
make clippy
make test
make e2e
make coverage
```

See the [language reference](language-reference/README.md), [architecture](architecture.md), and
[CLI reference](cli-reference.md) for the full contracts.
