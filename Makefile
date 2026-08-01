SHELL := /bin/bash
.DEFAULT_GOAL := help
.NOTPARALLEL:

RUST_TOOLCHAIN ?= 1.85.0
CARGO := cargo +$(RUST_TOOLCHAIN)
PACKAGE ?=
EXAMPLES ?=
PREVIEW_DIR ?= $(CURDIR)/examples-preview
PREVIEW_MAX_EDGE ?= 480
PREVIEW_FPS ?= 12
PORT ?= 8000

.PHONY: help doctor build check fmt fmt-check structure clippy lint test e2e
.PHONY: coverage-package coverage-packages coverage check-examples build-examples
.PHONY: serve-examples clean-examples check-language-docs verify
.PHONY: check-example-capabilities test-example-capabilities test-example-index
.PHONY: test-example-render-contracts

help: ## Show the available repository commands.
	@awk 'BEGIN {FS = ":.*## "; print "VEAC repository commands:\n"} /^[a-zA-Z0-9_.-]+:.*## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

doctor: ## Verify the pinned Rust, FFmpeg, and coverage tools.
	@command -v cargo >/dev/null
	@command -v ffmpeg >/dev/null
	@command -v ffprobe >/dev/null
	@command -v jq >/dev/null
	@command -v rg >/dev/null
	@$(CARGO) --version
	@rustc +$(RUST_TOOLCHAIN) --version
	@ffmpeg -version | head -n 1 | grep -E 'ffmpeg version 8\.0([.[:space:]]|$$)'
	@ffprobe -version | head -n 1 | grep -E 'ffprobe version 8\.0([.[:space:]]|$$)'
	@ffmpeg -hide_banner -filters 2>&1 | rg -q ' vidstabdetect +V->V '
	@ffmpeg -hide_banner -filters 2>&1 | rg -q ' vidstabtransform +V->V '
	@cargo llvm-cov --version | grep -E 'cargo-llvm-cov 0\.8\.4([.[:space:]]|$$)'

build: ## Build the workspace with every feature enabled.
	$(CARGO) build --workspace --all-features

check: check-example-capabilities ## Type-check all workspace targets and features.
	$(CARGO) check --workspace --all-targets --all-features

fmt: ## Format the Rust workspace.
	$(CARGO) fmt --all

fmt-check: ## Verify Rust formatting without changing files.
	$(CARGO) fmt --all -- --check

structure: ## Enforce file-size and production test-boundary rules.
	bash scripts/check-rust-structure.sh
	bash scripts/check-language-docs.sh

check-language-docs: ## Verify current authoring and canonical IR documentation contracts.
	bash scripts/check-language-docs.sh
	$(CARGO) test -p veac-lang --test language_docs_contract

clippy: ## Run Clippy with warnings denied.
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

lint: fmt-check structure check clippy ## Run every static repository gate.

test: test-example-capabilities ## Run the complete workspace test suite.
	$(CARGO) test --quiet --workspace --all-targets --all-features -- --test-threads=1

e2e: ## Run all real FFmpeg, ffprobe, workflow, and CLI E2E suites.
	$(CARGO) test -p veac-runtime --test render_e2e_tests -- --test-threads=1
	$(CARGO) test -p veac-runtime --test delivery_e2e_tests -- --test-threads=1
	$(CARGO) test -p veac-runtime --test probe_e2e_tests -- --test-threads=1
	$(CARGO) test -p veac-runtime --test workflow_e2e_tests -- --test-threads=1
	$(CARGO) test -p veac-cli --test cli_tests -- --test-threads=1

coverage-package: ## Gate one crate; pass PACKAGE=veac-ir.
	@test -n "$(PACKAGE)" || { echo "PACKAGE is required" >&2; exit 2; }
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) bash scripts/coverage.sh package $(PACKAGE)

coverage-packages: ## Gate production coverage independently for every crate.
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) bash scripts/coverage.sh packages

coverage: ## Gate workspace coverage and write target/coverage/lcov.info.
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) bash scripts/coverage.sh workspace target/coverage/lcov.info

verify: lint check-language-docs check-examples test e2e coverage coverage-packages ## Run every repository gate serially.

check-example-capabilities: ## Validate example coverage against stable capability IDs.
	bash scripts/check-example-capabilities.sh

test-example-capabilities: ## Exercise positive and negative catalog checks.
	bash scripts/tests/check-example-capabilities.sh

test-example-index: ## Verify example presentation and generated HTML contracts.
	bash scripts/tests/example-index-contracts.sh

test-example-render-contracts: ## Exercise preview and rendered-evidence shell contracts.
	bash scripts/tests/example-preview-contracts.sh
	bash scripts/tests/example-preview-fixture-contracts.sh
	bash scripts/tests/example-preview-provenance-contracts.sh
	bash scripts/tests/example-preview-build-flow-contracts.sh
	bash scripts/tests/render-evidence-contracts.sh
	bash scripts/tests/text-render-evidence-contracts.sh
	bash scripts/tests/timing-render-evidence-contracts.sh
	bash scripts/tests/media-smoke-render-evidence-contracts.sh
	bash scripts/tests/all-features-render-evidence-contracts.sh
	bash scripts/tests/agentsmesh-intro-render-evidence-contracts.sh
	bash scripts/tests/advanced-color-render-evidence-contracts.sh
	bash scripts/tests/mask-shape-render-evidence-contracts.sh
	bash scripts/tests/video-stabilization-render-evidence-contracts.sh
	bash scripts/tests/delivery-codec-render-evidence-contracts.sh

check-examples: check-example-capabilities test-example-capabilities test-example-index test-example-render-contracts ## Check catalog, contracts, compile, and format examples.
	$(CARGO) test -p veac-lang --test examples_authoring \
		--test examples_mechanism_evidence

build-examples: check-examples ## Render every example and generate a preview index.
	@unset VEAC_BIN; \
		VEAC_PREVIEW_MAX_EDGE=$(PREVIEW_MAX_EDGE) VEAC_PREVIEW_FPS=$(PREVIEW_FPS) \
		VEAC_EXAMPLES="$(EXAMPLES)" \
		RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) \
		bash scripts/build-examples.sh build "$(PREVIEW_DIR)"

serve-examples: ## Serve previously built previews on localhost.
	@test -f "$(PREVIEW_DIR)/index.html" || { echo "run 'make build-examples' first" >&2; exit 2; }
	python3 -m http.server "$(PORT)" --bind 127.0.0.1 --directory "$(PREVIEW_DIR)"

clean-examples: ## Remove generated example previews.
	bash scripts/build-examples.sh clean "$(PREVIEW_DIR)"
