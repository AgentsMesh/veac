use super::*;

#[test]
fn preconditions_are_checked_without_silent_noops() {
    let project = sample_project();
    let mut edit = batch(
        "op_preconditions",
        &project,
        vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_caption").unwrap(),
            enabled: false,
        }],
    );
    let source = project.project.sequences[0].tracks[1].clips[0]
        .source
        .clone();
    edit.preconditions = vec![
        Precondition::ClipExists {
            clip_id: ItemId::new("itm_caption").unwrap(),
        },
        Precondition::ClipSourceEquals {
            clip_id: ItemId::new("itm_caption").unwrap(),
            source: Box::new(source),
        },
        Precondition::TrackUnlocked {
            track_id: TrackId::new("trk_captions").unwrap(),
        },
    ];
    assert!(matches!(
        apply_edit_batch(&project, &edit),
        EditOutcome::Applied { .. }
    ));

    let failures = [
        Precondition::ClipExists {
            clip_id: ItemId::new("itm_missing").unwrap(),
        },
        Precondition::ClipSourceEquals {
            clip_id: ItemId::new("itm_caption").unwrap(),
            source: Box::new(ClipSource::Generated {
                generator: Generator::Silence,
            }),
        },
        Precondition::TrackUnlocked {
            track_id: TrackId::new("trk_missing").unwrap(),
        },
    ];
    for precondition in failures {
        let mut failing = edit.clone();
        failing.preconditions = vec![precondition];
        assert!(matches!(
            apply_edit_batch(&project, &failing),
            EditOutcome::Conflict { diagnostics, .. }
                if diagnostics[0].code == "PRECONDITION_FAILED"
        ));
    }

    let mut locked = project.clone();
    locked.project.sequences[0].tracks[1].state.locked = true;
    edit.preconditions = vec![Precondition::TrackUnlocked {
        track_id: TrackId::new("trk_captions").unwrap(),
    }];
    assert!(matches!(
        apply_edit_batch(&locked, &edit),
        EditOutcome::Conflict { diagnostics, .. }
            if diagnostics[0].code == "PRECONDITION_FAILED"
    ));
}
