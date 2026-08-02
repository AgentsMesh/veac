use super::*;

#[test]
fn locked_tracks_reject_track_controls_removal_and_indirect_material_replacement() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].state.locked = true;
    let track_id = TrackId::new("trk_video").unwrap();
    let mut material = project.project.materials[0].clone();
    material.source = MaterialSource::File {
        uri: "media/replacement.mp4".to_owned(),
    };
    let operations = vec![
        StructureEdit::RemoveTrack {
            track_id: track_id.clone(),
        },
        StructureEdit::SetTrackRouting {
            track_id: track_id.clone(),
            routing: TrackRouting::AudioBus {
                bus_id: BusId::new("bus_locked").unwrap(),
            },
        },
        StructureEdit::SetTrackKind {
            track_id: track_id.clone(),
            kind: TrackKind::Audio,
        },
        StructureEdit::SetTrackOrder {
            track_id: track_id.clone(),
            order: 4,
        },
        StructureEdit::SetTrackPlacementMode {
            track_id: track_id.clone(),
            placement_mode: PlacementMode::Free,
        },
        StructureEdit::SetTrackState {
            track_id: track_id.clone(),
            state: TrackState {
                muted: true,
                ..project.project.sequences[0].tracks[0].state
            },
        },
        StructureEdit::SetMaterial {
            material_id: material.id.clone(),
            material: Box::new(material),
        },
    ];
    for (index, edit) in operations.into_iter().enumerate() {
        let value = batch(
            &format!("op_locked_structure_{index}"),
            &project,
            vec![EditOperation::EditStructure { edit }],
        );
        assert_rejected(apply_edit_batch(&project, &value), "EDIT_REJECTED");
    }
}

#[test]
fn explicit_unlock_is_the_only_state_change_allowed_on_a_locked_track() {
    let mut project = sample_project();
    let state = &mut project.project.sequences[0].tracks[0].state;
    state.locked = true;
    let unlocked_state = TrackState {
        locked: false,
        ..*state
    };
    let edit = batch(
        "op_unlock_track",
        &project,
        vec![EditOperation::EditStructure {
            edit: StructureEdit::SetTrackState {
                track_id: TrackId::new("trk_video").unwrap(),
                state: unlocked_state,
            },
        }],
    );
    let unlocked = applied(apply_edit_batch(&project, &edit));
    assert!(!unlocked.project.sequences[0].tracks[0].state.locked);
}

#[test]
fn removing_a_sequence_with_any_locked_track_is_rejected() {
    let mut project = sample_project();
    let mut sequence = project.project.sequences[0].clone();
    sequence.id = SequenceId::new("seq_locked_aux").unwrap();
    sequence.name = "Locked auxiliary".to_owned();
    sequence.tracks[0].id = TrackId::new("trk_locked_aux").unwrap();
    sequence.tracks[0].clips.clear();
    sequence.tracks[0].state.locked = true;
    sequence.tracks.truncate(1);
    project.project.sequences.push(sequence);
    let edit = batch(
        "op_remove_locked_sequence",
        &project,
        vec![EditOperation::EditStructure {
            edit: StructureEdit::RemoveSequence {
                sequence_id: SequenceId::new("seq_locked_aux").unwrap(),
            },
        }],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
}
