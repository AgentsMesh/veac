use super::*;

#[test]
fn structure_edits_cover_insert_set_relink_and_reference_safe_remove() {
    let project = sample_project();
    let mut material = project.project.materials[1].clone();
    material.id = MaterialId::new("med_extra_font").unwrap();
    let mut sequence = project.project.sequences[0].clone();
    sequence.id = SequenceId::new("seq_aux").unwrap();
    sequence.name = "Aux".to_owned();
    sequence.tracks.clear();
    let mut track = project.project.sequences[0].tracks[1].clone();
    track.id = TrackId::new("trk_aux").unwrap();
    track.order = 0;
    track.clips.clear();
    let mut output = project.project.render_configs[0].clone();
    output.id = RenderConfigId::new("out_aux").unwrap();
    output.deliverables[0].id = DeliverableId::new("dlv_aux").unwrap();
    output.sequence_id = sequence.id.clone();
    let insert = batch(
        "op_structure_insert",
        &project,
        vec![
            structure(StructureEdit::InsertMaterial {
                material: Box::new(material.clone()),
                before_id: Some(MaterialId::new("med_video").unwrap()),
                after_id: None,
            }),
            structure(StructureEdit::InsertSequence {
                sequence: Box::new(sequence.clone()),
                relations: vec![],
                before_id: None,
                after_id: Some(SequenceId::new("seq_main").unwrap()),
            }),
            structure(StructureEdit::InsertTrack {
                sequence_id: sequence.id.clone(),
                track: Box::new(track.clone()),
                before_id: None,
                after_id: None,
            }),
            structure(StructureEdit::InsertOutput {
                output: Box::new(output.clone()),
                before_id: Some(RenderConfigId::new("out_main").unwrap()),
                after_id: None,
            }),
        ],
    );
    let inserted = applied(apply_edit_batch(&project, &insert));
    assert_eq!(inserted.project.materials[0].id, material.id);
    assert_eq!(inserted.project.render_configs[0].id, output.id);

    output.raster.as_mut().unwrap().width = 720;
    let settings = SequenceSettings {
        width: 720,
        ..sequence.settings.clone()
    };
    let set = batch(
        "op_structure_set",
        &inserted,
        vec![
            structure(StructureEdit::SetMaterial {
                material_id: material.id.clone(),
                material: Box::new(material.clone()),
            }),
            structure(StructureEdit::RelinkMaterial {
                material_id: material.id.clone(),
                source: MaterialSource::File {
                    uri: "fonts/relinked.ttf".to_owned(),
                },
                identity: None,
                probe: None,
            }),
            structure(StructureEdit::SetSequenceName {
                sequence_id: sequence.id.clone(),
                name: "Auxiliary".to_owned(),
            }),
            structure(StructureEdit::SetSequenceSettings {
                sequence_id: sequence.id.clone(),
                settings,
            }),
            structure(StructureEdit::SetTrackState {
                track_id: track.id.clone(),
                state: TrackState {
                    muted: true,
                    ..track.state
                },
            }),
            structure(StructureEdit::SetTrackKind {
                track_id: track.id.clone(),
                kind: TrackKind::Audio,
            }),
            structure(StructureEdit::SetTrackRouting {
                track_id: track.id.clone(),
                routing: TrackRouting::AudioBus {
                    bus_id: BusId::new("bus_dialog").unwrap(),
                },
            }),
            structure(StructureEdit::SetTrackOrder {
                track_id: track.id.clone(),
                order: 3,
            }),
            structure(StructureEdit::SetTrackPlacementMode {
                track_id: track.id.clone(),
                placement_mode: PlacementMode::Magnetic,
            }),
            structure(StructureEdit::SetOutput {
                output_id: output.id.clone(),
                output: Box::new(output.clone()),
            }),
            structure(StructureEdit::SetEntrySequence {
                sequence_id: sequence.id.clone(),
            }),
        ],
    );
    let updated = applied(apply_edit_batch(&inserted, &set));
    assert_eq!(updated.project.entry_sequence_id, sequence.id);

    let remove = batch(
        "op_structure_remove",
        &updated,
        vec![
            structure(StructureEdit::SetEntrySequence {
                sequence_id: SequenceId::new("seq_main").unwrap(),
            }),
            structure(StructureEdit::RemoveOutput {
                output_id: output.id.clone(),
            }),
            structure(StructureEdit::RemoveTrack {
                track_id: track.id.clone(),
            }),
            structure(StructureEdit::RemoveSequence {
                sequence_id: sequence.id.clone(),
            }),
            structure(StructureEdit::RemoveMaterial {
                material_id: material.id.clone(),
            }),
        ],
    );
    let removed = applied(apply_edit_batch(&updated, &remove));
    assert_eq!(removed.project.sequences.len(), 1);
    assert_eq!(removed.project.materials.len(), 2);
    assert_eq!(removed.project.render_configs.len(), 1);
}

#[test]
fn structure_edits_reject_references_bad_anchors_ids_and_locked_mutations() {
    let mut project = sample_project();
    let failures = vec![
        StructureEdit::RemoveMaterial {
            material_id: MaterialId::new("med_video").unwrap(),
        },
        StructureEdit::RemoveSequence {
            sequence_id: SequenceId::new("seq_main").unwrap(),
        },
        StructureEdit::InsertMaterial {
            material: Box::new(project.project.materials[1].clone()),
            before_id: Some(MaterialId::new("med_video").unwrap()),
            after_id: Some(MaterialId::new("med_font").unwrap()),
        },
        StructureEdit::SetOutput {
            output_id: RenderConfigId::new("out_main").unwrap(),
            output: Box::new({
                let mut value = project.project.render_configs[0].clone();
                value.id = RenderConfigId::new("out_other").unwrap();
                value
            }),
        },
    ];
    for (index, edit) in failures.into_iter().enumerate() {
        let value = batch(
            &format!("op_bad_structure_{index}"),
            &project,
            vec![structure(edit)],
        );
        assert_rejected(apply_edit_batch(&project, &value), "EDIT_REJECTED");
    }
    project.project.sequences[0].tracks[0].state.locked = true;
    let locked = batch(
        "op_locked_relink",
        &project,
        vec![structure(StructureEdit::RelinkMaterial {
            material_id: MaterialId::new("med_video").unwrap(),
            source: MaterialSource::File {
                uri: "media/new.mp4".to_owned(),
            },
            identity: project.project.materials[0].identity.clone(),
            probe: project.project.materials[0].probe.clone().map(Box::new),
        })],
    );
    assert_rejected(apply_edit_batch(&project, &locked), "EDIT_REJECTED");
}

fn structure(edit: StructureEdit) -> EditOperation {
    EditOperation::EditStructure { edit }
}
