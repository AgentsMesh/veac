# P1 Executable Evidence: Authoring And Composition

Each entry names a representative implementation, behavior/invalid-case test, and public integration
or decoded-output observation. The linked tests are entry points into broader module suites.

### P1-01
- P1-01 implementation: [project envelope](../../crates/veac-ir/src/model/project.rs)
- P1-01 verification: [canonical contract tests](../../crates/veac-ir/src/unit_tests/canonical_tests.rs)
- P1-01 rfc: [accepted RFC media model](../../crates/veac-ir/tests/rfc_media_example.rs)
- P1-01 observable: [canonical CLI integration](../../crates/veac-cli/tests/integration/canonical.rs)
### P1-02
- P1-02 implementation: [executable source parser](../../crates/veac-lang/src/program/parser/mod.rs)
- P1-02 verification: [public example conformance](../../crates/veac-lang/tests/examples_authoring.rs)
- P1-02 observable: [source CLI integration](../../crates/veac-cli/tests/integration/source.rs)
- P1-02 formatting: [source formatter contract](../../crates/veac-cli/src/unit_tests/frontend_tests.rs)
- P1-02 example: [self-contained executable source](../../examples/executable-mechanisms/main.veac)
### P1-03
- P1-03 implementation: [canonical timeline](../../crates/veac-ir/src/model/timeline.rs)
- P1-03 verification: [source-time resolution](../../crates/veac-plan/tests/source_time_resolution.rs)
- P1-03 observable: [source-time render E2E](../../crates/veac-runtime/tests/render_e2e/source_time.rs)
### P1-04
- P1-04 implementation: [atomic transaction](../../crates/veac-ir/src/edit/transaction.rs)
- P1-04 verification: [timeline edit tests](../../crates/veac-ir/src/unit_tests/edit_tests/timing_tests.rs)
- P1-04 observable: [edit CLI integration](../../crates/veac-cli/tests/integration/edit.rs)
### P1-05
- P1-05 implementation: [template proposal](../../crates/veac-template/src/proposal.rs)
- P1-05 verification: [fill-mode tests](../../crates/veac-template/tests/fill_modes.rs)
- P1-05 observable: [template FFmpeg binding E2E](../../crates/veac-cli/tests/integration/template_ffmpeg/binding.rs)
### P1-06
- P1-06 implementation: [track and routing model](../../crates/veac-ir/src/model/timeline.rs)
- P1-06 verification: [mechanism resolution](../../crates/veac-plan/tests/mechanism_resolution.rs)
- P1-06 observable: [multitrack audio E2E](../../crates/veac-runtime/tests/render_e2e/audio.rs)
### P1-07
- P1-07 implementation: [sequence resolver](../../crates/veac-plan/src/resolver/selection.rs)
- P1-07 verification: [resolution success tests](../../crates/veac-plan/tests/resolution_success.rs)
- P1-07 observable: [nested sequence E2E](../../crates/veac-runtime/tests/render_e2e/nested.rs)
### P1-08
- P1-08 implementation: [canonical relation model](../../crates/veac-ir/src/model/relation.rs)
- P1-08 verification: [linked split tests](../../crates/veac-ir/src/unit_tests/edit_tests/linked_split_tests.rs)
- P1-08 observable: [edit integration](../../crates/veac-cli/tests/integration/edit.rs)
### P1-09
- P1-09 implementation: [visual composition model](../../crates/veac-ir/src/model/properties/composition.rs)
- P1-09 verification: [composition resolution](../../crates/veac-plan/tests/composition_resolution.rs)
- P1-09 observable: [flip pixel E2E](../../crates/veac-runtime/tests/render_e2e/flip.rs)
### P1-10
- P1-10 implementation: [typed geometry](../../crates/veac-ir/src/model/properties/geometry.rs)
- P1-10 verification: [visual matrix codegen tests](../../crates/veac-codegen/src/unit_tests/emitter_tests/visual_matrix.rs)
- P1-10 observable: [layout matrix pixel E2E](../../crates/veac-runtime/tests/render_e2e/layout_matrix.rs)
### P1-11
- P1-11 implementation: [animatable values](../../crates/veac-ir/src/model/properties/animation.rs)
- P1-11 verification: [keyframe mutation tests](../../crates/veac-ir/src/unit_tests/edit_tests/keyframe_mutation_tests.rs)
- P1-11 observable: [animated composition E2E](../../crates/veac-runtime/tests/render_e2e/composition.rs)
### P1-12
- P1-12 implementation: [easing contract](../../crates/veac-ir/src/model/properties/easing.rs)
- P1-12 verification: [curve split tests](../../crates/veac-ir/src/unit_tests/edit_tests/curve_split_tests.rs)
- P1-12 observable: [Bezier equivalence E2E](../../crates/veac-runtime/tests/render_e2e/bezier.rs)
### P1-13
- P1-13 implementation: [compositing modes](../../crates/veac-ir/src/model/properties/composition.rs)
- P1-13 verification: [composition codegen tests](../../crates/veac-codegen/src/unit_tests/emitter_tests/composition.rs)
- P1-13 observable: [blend-mode pixel E2E](../../crates/veac-runtime/tests/render_e2e/composition_blend_modes.rs)
### P1-14
- P1-14 implementation: [mask and matte model](../../crates/veac-ir/src/model/properties/mask.rs)
- P1-14 verification: [composition validation](../../crates/veac-ir/src/unit_tests/validation_tests/composition_tests.rs)
- P1-14 observable: [mask-shape pixel E2E](../../crates/veac-runtime/tests/render_e2e/composition_mask_shapes.rs)
### P1-15
- P1-15 implementation: [card and shadow model](../../crates/veac-ir/src/model/properties/composition.rs)
- P1-15 language surface: [closed Domain operations](../../crates/veac-lang/src/program/domain_system/registry/surface.rs)
- P1-15 lowering: [Domain value to canonical visual surface](../../crates/veac-lang/src/program/executable/lower/visual/surface.rs)
- P1-15 verification: [executable visual lowering](../../crates/veac-lang/tests/executable_lowering/visual_v6.rs)
- P1-15 rejection: [typed transform diagnostics](../../crates/veac-lang/tests/executable_lowering/transform/errors.rs)
- P1-15 observable: [card render E2E](../../crates/veac-runtime/tests/render_e2e/composition_card.rs)
- P1-15 shadow observable: [shadow clipping pixel E2E](../../crates/veac-runtime/tests/render_e2e/composition_shadow_clipping.rs)
### P1-16
- P1-16 implementation: [text style model](../../crates/veac-ir/src/model/properties/text.rs)
- P1-16 verification: [text graphics validation](../../crates/veac-ir/src/unit_tests/validation_tests/text_graphics_tests.rs)
- P1-16 observable: [common text pipeline E2E](../../crates/veac-runtime/tests/render_e2e/text.rs)
### P1-17
- P1-17 implementation: [advanced text layout](../../crates/veac-ir/src/model/properties/text_layout.rs)
- P1-17 verification: [advanced text codegen](../../crates/veac-codegen/src/unit_tests/emitter_tests/text_advanced.rs)
- P1-17 observable: [Unicode/rich-layout E2E](../../crates/veac-runtime/tests/render_e2e/text_advanced.rs)
### P1-18
- P1-18 implementation: [text animation model](../../crates/veac-ir/src/model/properties/text_animation.rs)
- P1-18 verification: [text animation codegen](../../crates/veac-codegen/src/unit_tests/emitter_tests/text_animation.rs)
- P1-18 observable: [unit reveal/stagger E2E](../../crates/veac-runtime/tests/render_e2e/text_advanced.rs)
### P1-19
- P1-19 implementation: [caption document model](../../crates/veac-caption/src/model/document.rs)
- P1-19 verification: [cross-format tests](../../crates/veac-caption/tests/cross_format.rs)
- P1-19 observable: [caption delivery E2E](../../crates/veac-runtime/tests/delivery_e2e/professional_aux.rs)
### P1-20
- P1-20 implementation: [audio properties](../../crates/veac-ir/src/model/timeline.rs)
- P1-20 verification: [audio/color plan resolution](../../crates/veac-plan/tests/audio_color_resolution.rs)
- P1-20 observable: [audio timeline E2E](../../crates/veac-runtime/tests/render_e2e/audio.rs)
