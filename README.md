# VEAC — Video Editing as Code

![CI](https://github.com/AgentsMesh/veac/actions/workflows/ci.yml/badge.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

**Let AI agents edit videos by writing code, not clicking buttons.**

VEAC is a strongly typed, executable video-editing language designed for AI agents. `.veac` keeps
programmable intent and source spans; canonical JSON is the backend/interchange IR. Historical source
compatibility is not a design constraint.

```
executable .veac -> typed HIR -> verified Core v10 -> bounded graph build
                 -> frozen graph + temporal residualization -> canonical JSON IR
canonical IR -> ResolvedRenderPlan -> BackendBundle -> verified execution
```

## Quick Example

```veac
animate visual-opacity on clip(@hello, @main, @picture, @background) {
  clamp(progress * 2.0, 0.0, 1.0)
}

fn main(context: Context) -> Project {
  let background = item(
    identifier("background"), item_enabled(), during(0s, 4s),
    source_generated(generator_solid(#0f766eff)), source_timing_native()
  );
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let picture = visual_layer(
    identifier("picture"), 0, placement_free(), state, track_routing_default()
  ).with_item(background);
  let timeline = sequence(
    identifier("main"), "Hello, VEAC",
    sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000)
  ).with_layer(picture);
  project(identifier("hello"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
```

`main(Context) -> Project` is the video-program entry ABI; project and evidence hosts use typed entries described below. Root `animate` declarations residualize approved dynamic leaves.

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
- Executable modules, typed functions/closures, immutable collections, nominal structs/enums,
  static methods, inferred Effect/Stage, and transactional Project graph construction
- Strict canonical JSON, generated JSON Schema, reproducible hashes, and atomic typed edit batches
- Typed replaceable video/image slots and editable text fields with identity-bound probes, exact
  duration/crop policies, complete versioned fill requests, and reviewable atomic edit proposals
- Deterministic material identity/probe hydration with an explicit `--material-root` execution
  contract that keeps IR portable, before/after checks, and backend-neutral resolved render plans
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
curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | env VEAC_VERSION=v0.1.0 sh
```

### Build from Source

```bash
# Validated baseline: Rust 1.85.0 and FFmpeg/ffprobe 8.0
git clone https://github.com/AgentsMesh/veac.git
cd veac
make install
```

## Quick Start

```bash
veac check main.veac                              # Execute and validate without media I/O
veac fmt main.veac --check                        # Check canonical syntax-aware formatting
mkdir -p build && veac build main.veac --material-root . --emit-ir build/project.json # Detached IR
veac check-ir project.json                        # Validate canonical IR
veac edit project.json edit-batch.json --dry-run  # Preview one atomic typed edit
veac template propose project.json fill-request.json -o fill-edit.json # Fill typed slots
veac caption import captions.srt --format srt -o captions.json # Loss-aware sidecar import
veac otio export project.json -o timeline.otio --allow-lossy --loss-report otio-loss.json
veac plan build/project.json --material-root . --format json # Inspect detached IR
veac manifest project.json -o build.json          # Capture reproducible dependencies
veac bundle project.json --destination bundle     # Bundle reachable verified inputs
veac package-bindings bundle -o bindings.json     # Restore verified package bindings
veac relink project.json --search assets -o bindings.json     # Find SHA-256 matches
veac artifact inspect .veac-artifacts <key>        # Revalidate one cache entry
veac artifact materialize .veac-artifacts <key> artifact.bin # Copy verified payload
veac render build/project.json --material-root .   # Resolve assets without relocating the IR
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

The root Makefile keeps pinned development, test, coverage, and preview entrypoints in one place:

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
- [Executable Build](docs/language-reference/executable-build.md)、[工程工作区](docs/language-reference/project-workspaces.md) 与 [证据验收](docs/language-reference/evidence.md)
- [Architecture](docs/architecture.md)
- [Editing Capability Matrix and Roadmap](docs/capability-matrix-roadmap.md)

## Examples

Every example uses executable VEAC and passes build/canonical validation. The
[example catalog](examples/README.md) owns mechanism coverage; rendered previews live only in the
ignored `examples-preview/` directory.

## Why VEAC?

AI agents excel at file processing — reading, writing, and transforming text. But video editing has traditionally required GUI interaction that agents simply cannot perform. VEAC solves this by turning video editing into a **file-processing problem**:

- **Agent-first design**: The syntax is typed, structured, deterministic, and resource-bounded;
  stable IDs and checked edit batches let agents make small changes without rewriting a timeline
- **Programmable**: Functions and domain primitives construct intent; VEAC owns canonical validation and backend lowering
- **Verifiable**: `veac check` validates the file before rendering, giving agents fast feedback loops without waiting for video output
- **Single source format**: `.veac` files are plain text — agents can read, write, diff, and version-control them with standard file tools

## License

MIT — see [LICENSE](LICENSE). Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md).
