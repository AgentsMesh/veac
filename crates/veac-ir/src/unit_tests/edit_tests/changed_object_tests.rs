use super::*;

#[test]
fn clip_insert_remove_and_full_curve_replacement_report_the_complete_changed_tree() {
    let project = sample_project();
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    clip.id = ItemId::new("itm_insert_tree").unwrap();
    clip.record_range = range(600, 600);
    set_linear_start(&mut clip, time(600));
    clip.effects[0].id = EffectId::new("fx_insert_tree").unwrap();
    let keys = match &mut clip.visual.as_mut().unwrap().opacity {
        Animatable::Keyframes { keyframes } => keyframes,
        _ => unreachable!(),
    };
    keys[0].id = KeyframeId::new("kf_insert_start").unwrap();
    keys[1].id = KeyframeId::new("kf_insert_end").unwrap();
    let insert = batch(
        "op_insert_changed_tree",
        &project,
        vec![EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_video").unwrap(),
            clip: Box::new(clip),
            before_id: None,
            after_id: None,
        }],
    );
    let (inserted, changed) = outcome(apply_edit_batch(&project, &insert));
    assert!(changed.contains(&ChangedObjectId::Track {
        id: TrackId::new("trk_video").unwrap()
    }));
    assert!(changed.contains(&ChangedObjectId::Effect {
        id: EffectId::new("fx_insert_tree").unwrap()
    }));
    assert!(changed.contains(&ChangedObjectId::Keyframe {
        id: KeyframeId::new("kf_insert_start").unwrap()
    }));

    let remove = batch(
        "op_remove_changed_tree",
        &inserted,
        vec![EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_insert_tree").unwrap(),
        }],
    );
    let (_, removed) = outcome(apply_edit_batch(&inserted, &remove));
    assert!(removed.contains(&ChangedObjectId::Keyframe {
        id: KeyframeId::new("kf_insert_end").unwrap()
    }));

    let replacement = VisualProperties {
        opacity: Animatable::Keyframes {
            keyframes: vec![number_key("kf_replacement", 0, 1.0)],
        },
        ..project.project.sequences[0].tracks[0].clips[0]
            .visual
            .clone()
            .unwrap()
    };
    let set = batch(
        "op_replace_visual_tree",
        &project,
        vec![EditOperation::SetVisual {
            clip_id: ItemId::new("itm_video").unwrap(),
            visual: Some(replacement),
        }],
    );
    let (_, changed) = outcome(apply_edit_batch(&project, &set));
    for id in ["kf_opacity_start", "kf_opacity_end", "kf_replacement"] {
        assert!(changed.contains(&ChangedObjectId::Keyframe {
            id: KeyframeId::new(id).unwrap()
        }));
    }
}

#[test]
fn structure_inserts_report_new_objects_and_changed_containers() {
    let project = sample_project();
    let mut material = project.project.materials[1].clone();
    material.id = MaterialId::new("med_changed").unwrap();
    let mut sequence = project.project.sequences[0].clone();
    sequence.id = SequenceId::new("seq_changed").unwrap();
    sequence.name = "Changed".to_owned();
    sequence.tracks.clear();
    let mut track = project.project.sequences[0].tracks[1].clone();
    track.id = TrackId::new("trk_changed").unwrap();
    track.order = 20;
    track.clips.clear();
    let mut output = project.project.render_configs[0].clone();
    output.id = RenderConfigId::new("out_changed").unwrap();
    output.deliverables[0].id = DeliverableId::new("dlv_changed").unwrap();
    let operations = vec![
        StructureEdit::InsertMaterial {
            material: Box::new(material.clone()),
            before_id: None,
            after_id: None,
        },
        StructureEdit::InsertSequence {
            sequence: Box::new(sequence.clone()),
            relations: vec![],
            before_id: None,
            after_id: None,
        },
        StructureEdit::InsertTrack {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track: Box::new(track.clone()),
            before_id: None,
            after_id: None,
        },
        StructureEdit::InsertOutput {
            output: Box::new(output.clone()),
            before_id: None,
            after_id: None,
        },
    ]
    .into_iter()
    .map(|edit| EditOperation::EditStructure { edit })
    .collect();
    let (_, changed) = outcome(apply_edit_batch(
        &project,
        &batch("op_structure_changed", &project, operations),
    ));
    let expected = [
        ChangedObjectId::Project {
            id: project.project.id.clone(),
        },
        ChangedObjectId::Material { id: material.id },
        ChangedObjectId::Sequence { id: sequence.id },
        ChangedObjectId::Track { id: track.id },
        ChangedObjectId::Output { id: output.id },
    ];
    assert!(expected.iter().all(|value| changed.contains(value)));
}

fn outcome(value: EditOutcome) -> (ProjectEnvelope, Vec<ChangedObjectId>) {
    match value {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied, got {other:?}"),
    }
}
