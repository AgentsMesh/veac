#[path = "multicam_branch_tests/group_tests.rs"]
mod group_tests;
#[path = "multicam_branch_tests/switch_tests.rs"]
mod switch_tests;

use super::*;
use crate::test_support::multicam_project;

fn group_mut(project: &mut ProjectEnvelope) -> &mut MulticamGroup {
    &mut project.project.multicam_groups[0]
}

fn switches_mut(project: &mut ProjectEnvelope) -> &mut Vec<MulticamSwitch> {
    let ClipSource::Multicam { switches, .. } =
        &mut project.project.sequences[0].tracks[0].clips[0].source
    else {
        panic!()
    };
    switches
}

fn assert_multicam_code(project: &ProjectEnvelope, code: &str) {
    assert_code(&validation_codes(project), code);
}
