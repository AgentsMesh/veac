#[path = "effect_branch_tests/parameter_tests.rs"]
mod parameter_tests;
#[path = "effect_branch_tests/range_tests.rs"]
mod range_tests;

use super::*;

fn effect_mut(project: &mut ProjectEnvelope) -> &mut EffectInstance {
    &mut project.project.sequences[0].tracks[0].clips[0].effects[0]
}

fn effect_codes(project: &ProjectEnvelope) -> Vec<String> {
    validation_codes(project)
}

fn assert_effect_code(project: &ProjectEnvelope, code: &str) {
    assert_code(&effect_codes(project), code);
}
