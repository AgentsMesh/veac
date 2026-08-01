# P2 Provider-Application Evidence

Inference implementations are intentionally external. Evidence here proves the versioned contract,
target/source/evidence validation, deterministic proposal mapping, atomic application, and any local
render mechanism. “Contract” never means that VEAC bundles a model or claims model quality.

### P2-01
- P2-01 implementation: [speech contracts](../../crates/veac-provider/src/speech/mod.rs)
- P2-01 verification: [ASR proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/asr_tests.rs)
- P2-01 application: [translation proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/translation_tests.rs)
- P2-01 contract: [provider workflow E2E](../../crates/veac-runtime/tests/workflow_e2e/provider.rs)
### P2-02
- P2-02 implementation: [audio provider contract](../../crates/veac-provider/src/audio.rs)
- P2-02 verification: [separation proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/separation_tests.rs)
- P2-02 application: [audio filter E2E](../../crates/veac-runtime/tests/render_e2e/audio_filters.rs)
- P2-02 contract: [separation contract tests](../../crates/veac-provider/src/unit_tests/proposal_tests/separation_tests.rs)
### P2-03
- P2-03 implementation: [color provider contract](../../crates/veac-provider/src/vision/color.rs)
- P2-03 verification: [color proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/color_tests.rs)
- P2-03 application: [advanced color E2E](../../crates/veac-runtime/tests/render_e2e/color_processing.rs)
- P2-03 contract: [color contract tests](../../crates/veac-provider/src/unit_tests/proposal_tests/color_tests.rs)
### P2-04
- P2-04 implementation: [tracking contract](../../crates/veac-provider/src/vision/tracking.rs)
- P2-04 verification: [stabilization proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/stabilization_tests.rs)
- P2-04 application: [two-pass vidstab motion E2E](../../crates/veac-runtime/tests/render_e2e/stabilization.rs)
- P2-04 contract: [dynamic crop mapping tests](../../crates/veac-provider/src/unit_tests/proposal_tests/crop_dynamic_tests.rs)
### P2-05
- P2-05 implementation: [segmentation contract](../../crates/veac-provider/src/vision/segmentation.rs)
- P2-05 verification: [matte proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/matte_tests.rs)
- P2-05 application: [track-matte pixel E2E](../../crates/veac-runtime/tests/render_e2e/composition_matte.rs)
- P2-05 contract: [matte contract tests](../../crates/veac-provider/src/unit_tests/proposal_tests/matte_tests.rs)
### P2-06
- P2-06 implementation: [analysis contracts](../../crates/veac-provider/src/analysis/mod.rs)
- P2-06 verification: [annotation proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/annotation_tests.rs)
- P2-06 application: [annotation payload tests](../../crates/veac-ir/src/unit_tests/annotation_payload_tests.rs)
- P2-06 contract: [annotation contract tests](../../crates/veac-provider/src/unit_tests/proposal_tests/annotation_tests.rs)
- P2-06 observable: [media artifact workflow E2E](../../crates/veac-runtime/tests/workflow_e2e/media_artifacts.rs)
### P2-07
- P2-07 implementation: [reframe contract](../../crates/veac-provider/src/vision/reframe.rs)
- P2-07 verification: [crop proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/crop_tests.rs)
- P2-07 application: [provider crop-to-pixel E2E](../../crates/veac-runtime/tests/render_e2e/auto_reframe.rs)
- P2-07 contract: [crop tamper rejection](../../crates/veac-provider/src/unit_tests/proposal_tests/crop_dynamic_tamper_tests.rs)
### P2-08
- P2-08 implementation: [restoration contracts](../../crates/veac-provider/src/vision/restoration/removal.rs)
- P2-08 verification: [retouch proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/retouch_tests.rs)
- P2-08 application: [matte-backed Apply E2E](../../crates/veac-runtime/tests/render_e2e/composition_apply_matte.rs)
- P2-08 contract: [retouch rejection tests](../../crates/veac-provider/src/unit_tests/proposal_tests/retouch_rejection_tests.rs)
### P2-09
- P2-09 implementation: [multicam contract](../../crates/veac-provider/src/vision/multicam.rs)
- P2-09 verification: [multicam proposal tests](../../crates/veac-provider/src/unit_tests/proposal_tests/multicam_tests.rs)
- P2-09 application: [multicam source E2E](../../crates/veac-runtime/tests/render_e2e/multicam.rs)
- P2-09 contract: [multicam contract tests](../../crates/veac-provider/src/unit_tests/proposal_tests/multicam_tests.rs)
- P2-09 observable: [multicam E2E](../../crates/veac-runtime/tests/render_e2e/multicam.rs)
### P2-10
- P2-10 implementation: [provider runner](../../crates/veac-runtime/src/workflow/provider/runner.rs)
- P2-10 verification: [provider runner tests](../../crates/veac-runtime/src/workflow/provider/tests/runner_cases.rs)
- P2-10 application: [CLI workflow integration](../../crates/veac-cli/tests/integration/workflow.rs)
- P2-10 contract: [provider process E2E](../../crates/veac-runtime/tests/workflow_e2e/provider.rs)
