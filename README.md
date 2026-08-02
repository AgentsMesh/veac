# VEAC — Video Editing as Code

![CI](https://github.com/AgentsMesh/veac/actions/workflows/ci.yml/badge.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

**Let AI agents edit videos by writing code, not clicking buttons.**

VEAC is a declarative video-editing compiler designed for AI agents. The authoring language captures
intent with closed primitives and source spans; canonical JSON is the execution IR. Historical source
compatibility is not a design constraint.

```
.veac -> typed authoring AST -> canonical JSON IR
      -> ResolvedRenderPlan -> BackendBundle
      -> BundleExecutor (FFmpeg/write actions)
```

By abstracting video editing into a simple, declarative text format, any AI agent with file I/O capabilities can become a video editor — no mouse, no timeline UI, just code.

## Quick Example

```veac,compile
project hello {
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }
  entry sequence main;

  sequence main {
    layer visual picture {
      item title {
        source text {
          content "Hello, VEAC";
          style { font family "Inter"; size 72px; fill #ffffffff; }
        }
        record { at 0s; duration 4s; }
      }
    }
  }
}
```

## Features

- Declarative trim/split/overwrite/ripple/roll/slip/slide, snapping, linked AV edits, multicam cuts
  and multiview monitoring, reverse/freeze/loop/time-remap, transitions, and nested sequences
- Shared media/text/generated composition with explicit horizontal/vertical flip, 12 blend modes,
  cards, shadows, path/shape masks, track mattes, keying, scoped Apply pipelines, ordered effects, and
  arbitrary typed keyframe curves
- Unicode/BiDi text with rich spans, font fallback, horizontal/vertical/path layout, and
  whole/line/word/grapheme reveal, opacity, position, scale, and rotation animation
- Caption tracks and loss-aware SRT/WebVTT/ASS interchange; typed target-aware markers and provider
  analysis annotations with provenance
- Multitrack audio routing, automation, EQ/dynamics/loudness/ducking, plus provider-backed denoise,
  separation, TTS, and dubbing applications
- Ordered color-space/HSL/curve/wheel/LUT/RGB-matrix processing, scopes, and evidence-bound color match
- Stable typed IDs, separate record/source timing, rational project time, and canonical formatting
- Strict canonical JSON, generated JSON Schema, reproducible hashes, and atomic typed edit batches
- Typed replaceable video/image slots and editable text fields with identity-bound probes, exact
  duration/crop policies, complete versioned fill requests, and reviewable atomic edit proposals
- Deterministic material identity/probe hydration with before/after observation checks and
  backend-neutral resolved render plans
- Reproducible build manifests, identity-addressed packages, verified artifact caching, and relink
- Typed multi-deliverable output for alpha/HDR and MXF/DNxHR video, PNG/JPEG/TIFF/EXR sequences,
  SRT/WebVTT/ASS sidecars, PCM/compressed stems, scopes, portable execution policy, and one/two-pass
  execution; explicit hardware backends fail closed until device/upload bindings are modeled
- Original/prefer/require video/audio proxy substitution, exact render-segment reuse, and verified
  content-addressed artifact workflows
- Identity-bound `BackendBundle` resources, checkpoint v2 reuse, cross-process output locking,
  recoverable commit journals, no-follow filesystem mutation, and filesystem-aware alias defense
- Loss-aware OTIO projection with exact time conversion, typed annotation preservation, extension
  integrity, typed third-party bindings, and revision-checked atomic edit proposals
- Deterministic provider execution and evidence-bound atomic proposals for speech, analysis, tracking,
  stabilization, cutout, reframe, retouch/removal, color match, and automatic multicam sync
- Canonical plan inspection plus real FFmpeg execution through machine-local bindings

These are the currently implemented mechanisms, not a claim of feature parity with
mainstream editors. VEAC is evolving systematically toward mechanism parity with
products such as CapCut/Jianying; see the [capability matrix and
roadmap](docs/capability-matrix-roadmap.md). Template packages, bundled stock media, stickers, fonts,
music, effect packs, catalogs, marketplaces, and proprietary assets are intentionally outside that
target; typed template-slot representation and filling are part of the editing mechanism.

## Installation

### Quick Install (Mac / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | sh
```

Specify version:

```bash
VEAC_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | sh
```

### Build from Source

```bash
# Validated baseline: Rust 1.85.0 and FFmpeg/ffprobe 8.0
git clone https://github.com/AgentsMesh/veac.git
cd veac
cargo install --path crates/veac-cli
```

## Quick Start

```bash
veac check main.veac                              # Validate authoring source
veac fmt main.veac --check                        # Enforce canonical formatting
veac compile main.veac --emit-ir project.json     # Lower to canonical IR
veac check-ir project.json                        # Validate canonical IR
veac edit project.json edit-batch.json --dry-run  # Preview one atomic typed edit
veac template propose project.json fill-request.json -o fill-edit.json # Fill typed slots
veac caption import captions.srt --format srt -o captions.json # Loss-aware sidecar import
veac otio export project.json -o timeline.otio --allow-lossy --loss-report otio-loss.json
veac plan project.json --format json              # Inspect resolved plan
veac manifest project.json -o build.json          # Capture reproducible dependencies
veac package project.json --destination bundle    # Package reachable verified inputs
veac package-bindings bundle -o bindings.json     # Restore verified package bindings
veac relink project.json --search assets -o bindings.json     # Find SHA-256 matches
veac artifact inspect .veac-artifacts <key>        # Revalidate one cache entry
veac artifact materialize .veac-artifacts <key> artifact.bin # Copy verified payload
veac render project.json                           # Render every authored deliverable
veac probe assets/intro.mp4                       # Normalize media facts
veac schema --contract project > project.schema.json # Emit a public contract schema
```

`render` always produces every deliverable in the selected render config, using each authored
`file_name`. Files default to the project directory; `--destination <existing-directory>` relocates
the complete set without changing names. Verified per-task checkpoints live under
`<project-directory>/.veac-artifacts` and allow interrupted bundles to resume. Commits are atomic
per task, not across the complete bundle. The public render path always uses `emit_all` plus
`BundleExecutor`; there is no public single-command render bypass. Runtime holds deterministic
output-directory locks for the bundle, recovers durable prepared/committed journals before
rendering, uses descriptor-relative no-follow mutations, and probes destination case/Unicode alias
behavior. In safe Rust, only guarded runtime staging can issue the opaque FFmpeg invocation consumed
by an execution adapter. Bundle setup has a separate six-hour cap across recovery, resource snapshots,
and backend preflight. One deadline capped at six hours then spans each task's checkpoint lookup,
staging, FFmpeg process, output hashing, checkpoint publication, and journaled commit; separate
deliverables keep separate budgets. These guarantees require ordinary local filesystem locking and
durability semantics; they do not provide bundle-wide rollback. Failed rollback preserves its
prepared journal and backups for startup recovery. Exact render-segment promotion is accepted only
when the final file still matches the checkpoint digest and size and passes the same identity-bound
exact-delivery media postflight used for cache hits.

## Repository commands

The root Makefile keeps the pinned development, test, coverage, and preview
entrypoints in one place:

```bash
make help
make lint
make test
make e2e
make check-examples
make build-examples
make serve-examples
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [CLI Reference](docs/cli-reference.md)
- [Language Reference](docs/language-reference/)
- [Architecture](docs/architecture.md)
- [Editing Capability Matrix and Roadmap](docs/capability-matrix-roadmap.md)

## Examples

Every checked-in example uses the current authoring language and passes format,
lowering, and canonical validation. The [example catalog](examples/README.md)
is the source of truth for mechanisms and generated previews; rendered artifacts
live only in the ignored `examples-preview/` directory.

## Why VEAC?

AI agents excel at file processing — reading, writing, and transforming text. But video editing has traditionally required GUI interaction that agents simply cannot perform. VEAC solves this by turning video editing into a **file-processing problem**:

- **Agent-first design**: The target syntax is typed, structured, declarative, and non-Turing-complete;
  stable IDs and checked edit batches let agents make small changes without rewriting a timeline
- **Declarative**: Describe *what* the video should be, not *how* to process it — agents describe the desired outcome, VEAC handles the FFmpeg complexity
- **Verifiable**: `veac check` validates the file before rendering, giving agents fast feedback loops without waiting for video output
- **Single source format**: `.veac` files are plain text — agents can read, write, diff, and version-control them with standard file tools

## License

MIT — see [LICENSE](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
