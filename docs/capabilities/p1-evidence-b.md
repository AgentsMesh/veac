# P1 Executable Evidence: Processing And Delivery

These links distinguish public behavior from command-string construction. Backend rows point to
tests that inspect decoded media, ffprobe facts, artifact bytes, or guarded filesystem outcomes.

### P1-21
- P1-21 implementation: [audio automation model](../../crates/veac-ir/src/model/timeline.rs)
- P1-21 verification: [audio codegen tests](../../crates/veac-codegen/src/unit_tests/emitter_tests/audio.rs)
- P1-21 observable: [dynamics/loudness E2E](../../crates/veac-runtime/tests/render_e2e/audio_dynamics.rs)
### P1-22
- P1-22 implementation: [effect registry](../../crates/veac-ir/src/registry.rs)
- P1-22 verification: [animated effect codegen](../../crates/veac-codegen/src/unit_tests/emitter_tests/effects/animated.rs)
- P1-22 observable: [Apply stack E2E](../../crates/veac-runtime/tests/render_e2e/composition_apply_stack.rs)
### P1-23
- P1-23 implementation: [closed typed effect model](../../crates/veac-ir/src/model/effect.rs)
- P1-23 verification: [keying codegen tests](../../crates/veac-codegen/src/unit_tests/emitter_tests/keying.rs)
- P1-23 observable: [keying pixel E2E](../../crates/veac-runtime/tests/render_e2e/keying.rs)
### P1-24
- P1-24 implementation: [source-time model](../../crates/veac-ir/src/model/source_time.rs)
- P1-24 verification: [source-time plan tests](../../crates/veac-plan/tests/source_time_resolution.rs)
- P1-24 observable: [speed/repeat/pitch E2E](../../crates/veac-runtime/tests/render_e2e/source_time_policies.rs)
### P1-25
- P1-25 implementation: [source-time resolver](../../crates/veac-plan/src/resolver/source_time.rs)
- P1-25 verification: [source-time preflight](../../crates/veac-codegen/src/unit_tests/emitter_tests/source_time_preflight.rs)
- P1-25 observable: [time-remap E2E](../../crates/veac-runtime/tests/render_e2e/source_time.rs)
### P1-26
- P1-26 implementation: [transition model](../../crates/veac-ir/src/model/transition.rs)
- P1-26 verification: [transition contract validation](../../crates/veac-ir/src/unit_tests/validation_tests/transition_contract_tests.rs)
- P1-26 observable: [typed transition observability](../../crates/veac-runtime/tests/render_e2e/transition_typed_observability.rs)
- P1-26 contract: schema v10 publishes only explicit `centered` alignment and requires the authored duration to equal the adjacent items' exact true-overlap window.
### P1-27
- P1-27 model: [first-class Apply IR](../../crates/veac-ir/src/model/apply.rs)
- P1-27 implementation: [Apply resolver](../../crates/veac-plan/src/resolver/apply.rs)
- P1-27 verification: [executable Apply integration](../../crates/veac-lang/tests/executable_lowering/relation_apply_v6.rs)
- P1-27 observable: [Apply target/stack E2E](../../crates/veac-runtime/tests/render_e2e/composition_apply_stack.rs)
- P1-27 example: [closed Apply targets](../../examples/apply-scopes/main.veac)
### P1-28
- P1-28 implementation: [effect model](../../crates/veac-ir/src/model/effect.rs)
- P1-28 verification: [registry tests](../../crates/veac-ir/src/unit_tests/registry_tests.rs)
- P1-28 observable: [animated effects E2E](../../crates/veac-runtime/tests/render_e2e/effects.rs)
### P1-29
- P1-29 implementation: [generator model](../../crates/veac-ir/src/model/generator.rs)
- P1-29 verification: [generator validation](../../crates/veac-ir/src/unit_tests/validation_tests/generator_validation_tests.rs)
- P1-29 observable: [generated gradient E2E](../../crates/veac-runtime/tests/render_e2e/generated_gradients.rs)
- P1-29 example: [self-contained mechanisms source](../../examples/executable-mechanisms/main.veac)
### P1-30
- P1-30 implementation: [video output model](../../crates/veac-ir/src/model/output_video.rs)
- P1-30 verification: [output compatibility tests](../../crates/veac-ir/src/unit_tests/validation_tests/output_compat_tests.rs)
- P1-30 observable: [bitstream settings E2E](../../crates/veac-runtime/tests/render_e2e/output.rs)
- P1-30 example: [typed video delivery](../../examples/delivery-formats/main.veac)
### P1-31
- P1-31 implementation: [render budgets](../../crates/veac-ir/src/render_budget.rs)
- P1-31 verification: [untrusted-plan budget preflight](../../crates/veac-codegen/src/unit_tests/emitter_tests/render_budget_preflight.rs)
- P1-31 observable: [CLI fail-closed integration](../../crates/veac-cli/tests/integration/workflow_errors.rs)
### P1-32
- P1-32 implementation: [professional output model](../../crates/veac-ir/src/model/output_aux.rs)
- P1-32 implementation: [extended artifact model](../../crates/veac-ir/src/model/output_delivery.rs)
- P1-32 verification: [professional video codegen](../../crates/veac-codegen/src/unit_tests/emitter_tests/professional_video.rs)
- P1-32 verification: [auxiliary delivery codegen](../../crates/veac-codegen/src/unit_tests/emitter_tests/professional_aux.rs)
- P1-32 observable: [professional video E2E](../../crates/veac-runtime/tests/delivery_e2e/professional_video.rs)
- P1-32 observable: [auxiliary artifact E2E](../../crates/veac-runtime/tests/delivery_e2e/professional_aux.rs)
- P1-32 observable: [caption sidecar E2E](../../crates/veac-runtime/tests/delivery_e2e/captions.rs)
- P1-32 observable: [image sequence and scope E2E](../../crates/veac-runtime/tests/delivery_e2e/frames_scopes.rs)
- P1-32 observable: [MP3 and GIF E2E](../../crates/veac-runtime/tests/delivery_e2e/mp3_gif.rs)
- P1-32 observable: [still-image E2E](../../crates/veac-runtime/tests/delivery_e2e/still_formats.rs)
- P1-32 observable: [multi-rendition HLS E2E](../../crates/veac-runtime/tests/delivery_e2e/hls.rs)
- P1-32 example: [nine typed artifacts](../../examples/delivery-formats/main.veac)
### P1-33
- P1-33 implementation: [OTIO projection](../../crates/veac-otio/src/export.rs)
- P1-33 verification: [OTIO proposal tests](../../crates/veac-otio/src/unit_tests/proposal_tests.rs)
- P1-33 observable: [OTIO CLI integration](../../crates/veac-cli/tests/integration/otio.rs)
### P1-34
- P1-34 implementation: [probe normalization](../../crates/veac-runtime/src/asset/probe_json.rs)
- P1-34 verification: [probe parsing tests](../../crates/veac-runtime/src/asset/tests/parsing.rs)
- P1-34 observable: [stream-selection ffprobe E2E](../../crates/veac-runtime/tests/probe_e2e/stream_selection.rs)
### P1-35
- P1-35 implementation: [package model](../../crates/veac-artifact/src/package.rs)
- P1-35 verification: [package tests](../../crates/veac-artifact/src/unit_tests/package_tests.rs)
- P1-35 observable: [artifact CLI integration](../../crates/veac-cli/tests/integration/artifacts.rs)
### P1-36
- P1-36 implementation: [artifact cache](../../crates/veac-artifact/src/cache.rs)
- P1-36 verification: [cache safety tests](../../crates/veac-artifact/src/unit_tests/cache_safety_tests.rs)
- P1-36 observable: [cache CLI integration](../../crates/veac-cli/tests/integration/artifact_cache.rs)
### P1-37
- P1-37 implementation: [artifact workflow](../../crates/veac-artifact/src/workflow.rs)
- P1-37 contract: FFmpeg derives proxy, optical-flow, and source-segment artifacts; external analyzers publish closed typed result envelopes through explicit analysis ingestion. Analysis is not advertised as a local FFmpeg derivation.
- P1-37 verification: [render-segment guard tests](../../crates/veac-artifact/src/render_segment/guard_tests.rs)
- P1-37 observable: [proxy substitution E2E](../../crates/veac-runtime/tests/render_e2e/proxy_substitution.rs)
- P1-37 typed analysis observable: [analysis ingestion E2E](../../crates/veac-cli/tests/integration/analysis_ingestion.rs)
### P1-38
- P1-38 implementation: [CLI dispatch](../../crates/veac-cli/src/main.rs)
- P1-38 verification: [command tests](../../crates/veac-cli/src/unit_tests/command_tests.rs)
- P1-38 observable: [source-to-real-render example E2E](../../crates/veac-cli/tests/integration/executable_example.rs)
### P1-39
- P1-39 implementation: [descriptor-relative recovery](../../crates/veac-runtime/src/executor/staging/recovery.rs)
- P1-39 verification: [recovery safety contracts](../../crates/veac-runtime/src/executor/staging/tests/recovery_safety_contracts.rs)
- P1-39 observable: [checkpoint delivery E2E](../../crates/veac-runtime/tests/delivery_e2e/checkpoint.rs)
### P1-40
- P1-40 implementation: [typed function resolver](../../crates/veac-lang/src/program/resolve/functions.rs)
- P1-40 implementation: [compiled expression core](../../crates/veac-lang/src/program/expression/compile/mod.rs)
- P1-40 verification: [control-flow integration](../../crates/veac-lang/tests/programming_language_control_flow.rs)
- P1-40 Temporal verification: [authored Temporal integration](../../crates/veac-lang/tests/executable_authored_temporal.rs)
- P1-40 source editing: [source transaction](../../crates/veac-lang/src/program/source_transaction.rs)
- P1-40 observable: [programming language example](../../examples/programming-language/main.veac)
- P1-40 source-of-truth transaction: [revision-bound edit batch](../../examples/programming-language/source-edit.json)
### P1-41
- P1-41 implementation: [typed plugin lowering](../../crates/veac-lang/src/program/executable/lower/effect/video.rs)
- P1-41 verification: [descriptor and canonical effect tests](../../crates/veac-lang/tests/executable_lowering/plugin_effect.rs)
- P1-41 observable: [color-to-monochrome example](../../examples/video-effects/main.veac)
### P1-42
- P1-42 implementation: [curve residualization](../../crates/veac-lang/src/program/expression/residual/instruction/call/builtin/curve.rs)
- P1-42 verification: [authored curve integration](../../crates/veac-lang/tests/executable_authored_temporal/curves.rs)
- P1-42 observable: [curve-driven transform example](../../examples/transforms-and-animation/main.veac)
