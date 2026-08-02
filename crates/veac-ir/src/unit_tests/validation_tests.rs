#[path = "validation_tests/apply_crossing_tests.rs"]
mod apply_crossing_tests;
#[path = "validation_tests/apply_json_tests.rs"]
mod apply_json_tests;
#[path = "validation_tests/apply_matte_tests.rs"]
mod apply_matte_tests;
#[path = "validation_tests/apply_pipeline_tests.rs"]
mod apply_pipeline_tests;
#[path = "validation_tests/apply_target_tests.rs"]
mod apply_target_tests;
#[path = "validation_tests/audio_processing_fixture.rs"]
mod audio_processing_fixture;
#[path = "validation_tests/audio_processing_tests.rs"]
mod audio_processing_tests;
#[path = "validation_tests/audio_processor_identity_tests.rs"]
mod audio_processor_identity_tests;
#[path = "validation_tests/caption_speaker_tests.rs"]
mod caption_speaker_tests;
#[path = "validation_tests/color_matrix_tests.rs"]
mod color_matrix_tests;
#[path = "validation_tests/color_pipeline_tests.rs"]
mod color_pipeline_tests;
#[path = "validation_tests/color_space_tests.rs"]
mod color_space_tests;
#[path = "validation_tests/composition_tests.rs"]
mod composition_tests;
#[path = "validation_tests/effect_branch_tests.rs"]
mod effect_branch_tests;
#[path = "validation_tests/effect_tests.rs"]
mod effect_tests;
#[path = "validation_tests/generator_validation_tests.rs"]
mod generator_validation_tests;
#[path = "validation_tests/invalid_timebase_tests.rs"]
mod invalid_timebase_tests;
#[path = "validation_tests/matte_activity_tests.rs"]
mod matte_activity_tests;
#[path = "validation_tests/matte_dependency_tests.rs"]
mod matte_dependency_tests;
#[path = "validation_tests/media_domain_tests.rs"]
mod media_domain_tests;
#[path = "validation_tests/multicam_branch_tests.rs"]
mod multicam_branch_tests;
#[path = "validation_tests/multicam_tests.rs"]
mod multicam_tests;
#[path = "validation_tests/output_aux_tests.rs"]
mod output_aux_tests;
#[path = "validation_tests/output_collision_tests.rs"]
mod output_collision_tests;
#[path = "validation_tests/output_compat_tests.rs"]
mod output_compat_tests;
#[path = "validation_tests/output_contract_tests.rs"]
mod output_contract_tests;
#[path = "validation_tests/output_tests.rs"]
mod output_tests;
#[path = "validation_tests/probe_tests.rs"]
mod probe_tests;
#[path = "validation_tests/project_tests.rs"]
mod project_tests;
#[path = "validation_tests/render_budget_identity_tests.rs"]
mod render_budget_identity_tests;
#[path = "validation_tests/render_budget_tests.rs"]
mod render_budget_tests;
#[path = "validation_tests/sidechain_activity_tests.rs"]
mod sidechain_activity_tests;
#[path = "validation_tests/source_tests.rs"]
mod source_tests;
#[path = "validation_tests/source_time_tests.rs"]
mod source_time_tests;
#[path = "validation_tests/template_tests.rs"]
mod template_tests;
#[path = "validation_tests/text_budget_tests.rs"]
mod text_budget_tests;
#[path = "validation_tests/text_geometry_tests.rs"]
mod text_geometry_tests;
#[path = "validation_tests/text_graphics_tests.rs"]
mod text_graphics_tests;
#[path = "validation_tests/timeline_tests.rs"]
mod timeline_tests;
#[path = "validation_tests/transition_contract_tests.rs"]
mod transition_contract_tests;
#[path = "validation_tests/visual_tests.rs"]
mod visual_tests;

use crate::{test_support::sample_project, *};

fn validation_codes(project: &ProjectEnvelope) -> Vec<String> {
    validate(project)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn assert_code(codes: &[String], code: &str) {
    assert!(
        codes.iter().any(|actual| actual == code),
        "missing {code}: {codes:?}"
    );
}

#[test]
fn sample_project_is_valid() {
    validate(&sample_project()).unwrap();
}
