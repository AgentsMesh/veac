# Contributing to VEAC

Thank you for your interest in contributing to VEAC (Video Editing as Code)! This guide will help you get started.

## Development Environment Setup

### Prerequisites

- **Rust toolchain 1.85.0** — install via [rustup](https://rustup.rs/)
- **FFmpeg and ffprobe 8.0** — both executables must resolve on `PATH`

### Getting Started

```bash
git clone https://github.com/AgentsMesh/veac.git
cd veac
cargo build
```

## Project Architecture

VEAC is organized as a Cargo workspace with eleven crates:

| Crate | Responsibility |
|---|---|
| **veac-ir** | Versioned canonical NLE model, JCS JSON, validation, and typed edit batches |
| **veac-plan** | Backend-neutral resolution from canonical IR to immutable render plans |
| **veac-artifact** | Content-addressed artifacts, build manifests, packaging, and relinking |
| **veac-provider** | Versioned deterministic contracts for external analysis/model providers |
| **veac-caption** | Loss-aware SRT, WebVTT, and ASS interchange with canonical timed captions |
| **veac-otio** | Loss-aware OpenTimelineIO projection, typed bindings, and edit proposals |
| **veac-template** | Typed template-fill contracts and reviewable atomic edit proposals |
| **veac-lang** | Agent authoring lexer, parser, semantic analysis, and lowering |
| **veac-codegen** | Resolved render plan to identity-bound, typed multi-deliverable `BackendBundle` tasks |
| **veac-runtime** | Identity-stable probing plus guarded bundle/checkpoint v2 execution and progress parsing |
| **veac-cli** | Canonical CLI (`compile`, `edit`, `caption`, `otio`, provider/artifact workflows, `plan`, and `render`) |

The public render architecture is fixed: v2 source or `EditBatch` -> canonical JSON IR ->
`ResolvedRenderPlan` -> `BackendBundle` -> `BundleExecutor` running structured FFmpeg/write actions.
Do not add legacy compatibility models or a public single-`BackendCommand` render bypass.

## Code Style

All code must pass formatting and lint checks before merging:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Please run both commands before submitting a pull request.

Every controlled implementation, test, documentation, example, specification, standard-library,
and tooling file must remain strictly below 200 lines. This includes package manifests and
extensionless `veac.package.lock` files under `stdlib/`. Split Rust modules along coherent
responsibilities using ordinary submodules; `include!` and simultaneous `name.rs`/`name/mod.rs`
module layouts are rejected by CI. Run the same structural check locally with:

```bash
bash scripts/check-rust-structure.sh
```

## Testing

Run the full test suite with:

```bash
cargo test --workspace --all-targets
```

FFmpeg-backed integration tests generate small synthetic fixtures at runtime, so no media assets
need to be checked into the repository. FFmpeg and ffprobe must both be available on `PATH`.

Install `cargo-llvm-cov` and verify the same strictly-greater-than-95% production line
and function coverage gates used by CI. The package profile gives every dynamically
discovered workspace crate an isolated target containing its library and binary unit
targets. The workspace profile uses another fresh target and runs `--all-targets`, so
black-box integration and FFmpeg E2E targets execute before LCOV and the workspace and
per-crate gates are produced.

```bash
cargo install cargo-llvm-cov --version 0.8.4 --locked
bash scripts/coverage.sh packages
bash scripts/coverage.sh workspace target/coverage/lcov.info
```

Both reports exclude files in explicit test-only paths such as `tests`, `unit_tests`,
and `test_support`; this prevents test helpers from inflating the production function
percentage. Tests in those paths still compile and run in their owning profile.
Production files may declare test modules, but must not contain inline `#[test]`
functions or `#[cfg(test)] fn` helpers; the structure script enforces that boundary.

CI also schedules a dedicated FFmpeg job that reruns the CLI and render end-to-end
targets against real `ffmpeg`/`ffprobe` binaries. Run the same targets locally with:

```bash
cargo test -p veac-cli --test cli_tests
cargo test -p veac-runtime --test probe_e2e_tests
cargo test -p veac-runtime --test render_e2e_tests
cargo test -p veac-runtime --test workflow_e2e_tests
cargo test -p veac-runtime --test delivery_e2e_tests
```

When adding a mechanism, include focused unit tests for validation and code generation
plus an integration test that executes the CLI or FFmpeg path. Coverage must come from
exercising behavior; production modules must not be excluded merely to satisfy the
threshold.

Changes to bundle execution must preserve the typed protected-resource contract. Codegen emits
UTF-8-path-sorted `{ path, expected SHA-256 }` resources; runtime verifies them before tool
fingerprinting, around resume/action work, and after checkpoint store before commit. Tests must
cover mutation/no-side-effect behavior, cache identity, dynamic output collisions, and malformed
two-pass/source-time plans. Per-task commit is not a bundle-wide or cross-process transaction; do
not weaken output locks, crash-recovery journaling, or future-path case/Unicode alias handling
without updating their adversarial tests and documented guarantees.

## Pull Request Workflow

1. **Fork** the repository and clone your fork locally.
2. **Create a branch** from `main` for your change:
   ```bash
   git checkout -b my-feature
   ```
3. **Make your changes** in small, focused commits with clear messages.
4. **Run checks** before pushing:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-targets
   bash scripts/check-rust-structure.sh
   bash scripts/coverage.sh packages
   bash scripts/coverage.sh workspace target/coverage/lcov.info
   cargo test -p veac-cli --test cli_tests
   cargo test -p veac-runtime --test probe_e2e_tests
   cargo test -p veac-runtime --test render_e2e_tests
   cargo test -p veac-runtime --test workflow_e2e_tests
   cargo test -p veac-runtime --test delivery_e2e_tests
   ```
5. **Push** your branch and open a **Pull Request** against `main`.
6. Respond to any **review feedback** and update your PR as needed.

## Reporting Issues

When opening an issue, please include:

- A clear and descriptive title.
- Steps to reproduce the problem (including a minimal `.veac` file if applicable).
- Expected behavior vs. actual behavior.
- Your environment details (OS, Rust version, FFmpeg version).
- Any relevant error messages or log output.

## License

By contributing to VEAC, you agree that your contributions will be licensed under the [MIT License](LICENSE).
