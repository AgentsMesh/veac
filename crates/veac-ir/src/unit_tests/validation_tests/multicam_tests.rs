use super::*;
use crate::test_support::{multicam_project, range, time};

#[test]
fn synchronized_multicam_group_and_contiguous_switches_validate() {
    let mut project = multicam_project();
    validate(&project).unwrap();

    project.project.multicam_groups[0].sync.basis = MulticamSyncBasis::Timecode;
    validate(&project).unwrap();

    project.project.multicam_groups[0].sync.basis = MulticamSyncBasis::Audio;
    validate(&project).unwrap();
}

#[test]
fn multicam_group_rejects_order_reference_offset_kind_and_audio_failures() {
    let mut project = multicam_project();
    project.project.multicam_groups[0].angles.swap(0, 1);
    project.project.multicam_groups[0].sync.reference_angle_id =
        MulticamAngleId::new("ang_missing").unwrap();
    project.project.multicam_groups[0].angles[0].source_offset = time(-1);
    project.project.multicam_groups[0].angles[1].material_id = MaterialId::new("med_font").unwrap();
    let codes = validation_codes(&project);
    for code in [
        "MULTICAM_ANGLE_ORDER",
        "MULTICAM_REFERENCE_ANGLE_NOT_FOUND",
        "MULTICAM_SOURCE_OFFSET",
        "MULTICAM_VIDEO_MATERIAL_NOT_FOUND",
    ] {
        assert_code(&codes, code);
    }

    let mut audio = multicam_project();
    audio.project.multicam_groups[0].sync.basis = MulticamSyncBasis::Audio;
    audio.project.materials[2]
        .probe
        .as_mut()
        .unwrap()
        .selected_audio_stream = None;
    assert_code(&validation_codes(&audio), "MULTICAM_SYNC_AUDIO_MISSING");
}

#[test]
fn multicam_switches_reject_missing_groups_angles_gaps_and_overlaps() {
    let mut project = multicam_project();
    let ClipSource::Multicam { switches, .. } =
        &mut project.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    switches[0].range = range(0, 350);
    switches[1].range = range(300, 300);
    switches[1].angle_id = MulticamAngleId::new("ang_missing").unwrap();
    let codes = validation_codes(&project);
    assert_code(&codes, "MULTICAM_SWITCH_PARTITION");
    assert_code(&codes, "MULTICAM_SWITCH_ANGLE_NOT_FOUND");

    let mut missing = multicam_project();
    let ClipSource::Multicam { group_id, .. } =
        &mut missing.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    *group_id = MulticamGroupId::new("mcg_missing").unwrap();
    assert_code(&validation_codes(&missing), "MULTICAM_GROUP_NOT_FOUND");
}
