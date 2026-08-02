use super::*;
use crate::test_support::time;

#[test]
fn group_and_angle_collections_require_counts_unique_ids_and_strict_order() {
    let mut too_small = multicam_project();
    group_mut(&mut too_small).angles.pop();
    assert_multicam_code(&too_small, "MULTICAM_ANGLE_COUNT");

    let mut duplicate_angle = multicam_project();
    let first = group_mut(&mut duplicate_angle).angles[0].id.clone();
    group_mut(&mut duplicate_angle).angles[1].id = first;
    let codes = validation_codes(&duplicate_angle);
    assert_code(&codes, "DUPLICATE_MULTICAM_ANGLE_ID");
    assert_code(&codes, "MULTICAM_ANGLE_ORDER");

    let mut duplicate_group = multicam_project();
    let duplicate = group_mut(&mut duplicate_group).clone();
    duplicate_group.project.multicam_groups.push(duplicate);
    let codes = validation_codes(&duplicate_group);
    assert_code(&codes, "DUPLICATE_MULTICAM_GROUP_ID");
    assert_code(&codes, "MULTICAM_GROUP_ORDER");

    let mut reversed = multicam_project();
    let mut earlier = group_mut(&mut reversed).clone();
    earlier.id = MulticamGroupId::new("mcg_alpha").unwrap();
    reversed.project.multicam_groups.push(earlier);
    assert_multicam_code(&reversed, "MULTICAM_GROUP_ORDER");
}

#[test]
fn group_and_angle_ids_and_offsets_are_validated_at_each_level() {
    let mut invalid_group = multicam_project();
    group_mut(&mut invalid_group).id = serde_json::from_str("\"bad\"").unwrap();
    assert_multicam_code(&invalid_group, "INVALID_ID");

    let mut invalid_angle = multicam_project();
    group_mut(&mut invalid_angle).angles[0].id = serde_json::from_str("\"bad\"").unwrap();
    assert_multicam_code(&invalid_angle, "INVALID_ID");

    for offset in [
        RationalTime {
            value: 0,
            timescale: 1,
        },
        RationalTime {
            value: 0,
            timescale: 0,
        },
        time(-1),
    ] {
        let mut project = multicam_project();
        group_mut(&mut project).angles[0].source_offset = offset;
        assert_multicam_code(&project, "MULTICAM_SOURCE_OFFSET");
    }
}

#[test]
fn audio_sync_distinguishes_known_missing_audio_from_unknown_probe_facts() {
    let mut known_missing = multicam_project();
    group_mut(&mut known_missing).sync.basis = MulticamSyncBasis::Audio;
    known_missing.project.materials[0]
        .probe
        .as_mut()
        .unwrap()
        .selected_audio_stream = None;
    assert_multicam_code(&known_missing, "MULTICAM_SYNC_AUDIO_MISSING");

    let mut unknown = multicam_project();
    group_mut(&mut unknown).sync.basis = MulticamSyncBasis::Audio;
    unknown.project.materials[0].probe = None;
    unknown.project.materials[2].probe = None;
    validate(&unknown).unwrap();
}
