use super::*;
use crate::test_support::{multicam_project, range, time};

#[test]
fn multicam_switch_and_sync_edits_are_atomic_typed_and_report_changes() {
    let project = multicam_project();
    let switches = vec![MulticamSwitch {
        angle_id: MulticamAngleId::new("ang_wide").unwrap(),
        range: range(0, 600),
    }];
    let mut angles = project.project.multicam_groups[0].angles.clone();
    angles[1].source_offset = time(120);
    let edit = batch(
        "op_multicam",
        &project,
        vec![
            EditOperation::SetMulticamSwitches {
                clip_id: ItemId::new("itm_video").unwrap(),
                switches: switches.clone(),
            },
            EditOperation::SetMulticamGroup {
                group_id: MulticamGroupId::new("mcg_interview").unwrap(),
                sync: MulticamSync {
                    basis: MulticamSyncBasis::Timecode,
                    reference_angle_id: MulticamAngleId::new("ang_close").unwrap(),
                },
                angles: angles.clone(),
            },
        ],
    );
    let EditOutcome::Applied {
        project,
        changed_objects,
        ..
    } = apply_edit_batch(&project, &edit)
    else {
        panic!("multicam edit must apply")
    };
    let ClipSource::Multicam {
        switches: current, ..
    } = &project.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    assert_eq!(current, &switches);
    assert_eq!(project.project.multicam_groups[0].angles, angles);
    assert!(changed_objects.contains(&ChangedObjectId::MulticamGroup {
        id: MulticamGroupId::new("mcg_interview").unwrap()
    }));
}

#[test]
fn multicam_edits_reject_locked_wrong_kind_and_invalid_partitions() {
    let mut locked = multicam_project();
    locked.project.sequences[0].tracks[0].state.locked = true;
    let group = &locked.project.multicam_groups[0];
    let edit = batch(
        "op_multicam_locked",
        &locked,
        vec![EditOperation::SetMulticamGroup {
            group_id: group.id.clone(),
            sync: group.sync.clone(),
            angles: group.angles.clone(),
        }],
    );
    assert_rejected(apply_edit_batch(&locked, &edit), "EDIT_REJECTED");

    let project = multicam_project();
    let invalid = batch(
        "op_multicam_invalid",
        &project,
        vec![EditOperation::SetMulticamSwitches {
            clip_id: ItemId::new("itm_video").unwrap(),
            switches: vec![MulticamSwitch {
                angle_id: MulticamAngleId::new("ang_wide").unwrap(),
                range: range(0, 300),
            }],
        }],
    );
    assert_rejected(
        apply_edit_batch(&project, &invalid),
        "MULTICAM_SWITCH_PARTITION",
    );
}
