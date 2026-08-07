use super::*;

use crate::test_support::{range, sample_project, time};

#[test]
fn ownership_edits_clear_aggregate_and_inserted_entity_authorship() {
    let mut project = authored_project();
    let existing_authorship = entity(&["project", "sequence", "video", "existing"]);
    project.project.sequences[0].tracks[0].clips[0].authorship = Some(existing_authorship.clone());
    project.project.materials[0].authorship = Some(entity(&["project", "materials", "video"]));

    let mut track = project.project.sequences[0].tracks[1].clone();
    track.id = TrackId::new("trk_inserted").unwrap();
    track.order = 11;
    track.clips[0].id = ItemId::new("itm_inserted").unwrap();
    track.clips[0].record_range = range(600, 300);
    track.clips[0].authorship = Some(entity(&["forged", "inserted", "clip"]));
    let mut material = project.project.materials[0].clone();
    material.source = MaterialSource::File {
        uri: "media/replaced.mp4".to_owned(),
    };
    let annotation = Annotation {
        id: AnnotationId::new("ann_inserted").unwrap(),
        target: AnnotationTarget::Project,
        span: AnnotationSpan::Untimed,
        payload: AnnotationPayload::Marker {
            label: "inserted".to_owned(),
            color: None,
        },
        provenance: None,
    };
    let operations = vec![
        EditOperation::EditStructure {
            edit: StructureEdit::InsertTrack {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track: Box::new(track),
                before_id: None,
                after_id: None,
            },
        },
        EditOperation::InsertAnnotation {
            annotation: Box::new(annotation),
        },
        EditOperation::EditStructure {
            edit: StructureEdit::SetMaterial {
                material_id: material.id.clone(),
                material: Box::new(material),
            },
        },
    ];

    let updated = applied(apply_edit_batch(
        &project,
        &batch("op_authorship_ownership", &project, operations),
    ));
    assert!(updated.project.authorship.is_none());
    assert!(updated.project.sequences[0].authorship.is_none());
    assert!(updated.project.materials[0].authorship.is_none());
    assert_eq!(
        updated.project.sequences[0].tracks[0].clips[0].authorship,
        Some(existing_authorship)
    );
    assert!(updated.project.sequences[0].tracks[2].clips[0]
        .authorship
        .is_none());
}

#[test]
fn split_strips_only_the_new_fragment_and_leaf_edits_preserve_origin() {
    let mut project = authored_project();
    let clip_origin = entity(&["project", "sequence", "video", "clip"]);
    project.project.sequences[0].tracks[0].clips[0].authorship = Some(clip_origin.clone());
    let split = EditOperation::SplitClip {
        clip_id: ItemId::new("itm_video").unwrap(),
        at: time(300),
        right_clip_id: ItemId::new("itm_video_right").unwrap(),
        relation_fragments: Vec::new(),
    };
    let split = applied(apply_edit_batch(
        &project,
        &batch("op_authorship_split", &project, vec![split]),
    ));
    let clips = &split.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].authorship, Some(clip_origin.clone()));
    assert!(clips[1].authorship.is_none());
    assert!(split.project.sequences[0].authorship.is_some());

    let leaf = EditOperation::SetClipEnabled {
        clip_id: ItemId::new("itm_video").unwrap(),
        enabled: false,
    };
    let leaf = applied(apply_edit_batch(
        &split,
        &batch("op_authorship_leaf", &split, vec![leaf]),
    ));
    assert_eq!(
        leaf.project.sequences[0].tracks[0].clips[0].authorship,
        Some(clip_origin)
    );
    assert!(leaf.project.sequences[0].authorship.is_some());
}

#[test]
fn exact_noop_material_replacement_preserves_authorship() {
    let mut project = authored_project();
    project.project.materials[0].authorship = Some(entity(&["project", "materials", "video"]));
    let material = project.project.materials[0].clone();
    let operation = EditOperation::EditStructure {
        edit: StructureEdit::SetMaterial {
            material_id: material.id.clone(),
            material: Box::new(material.clone()),
        },
    };
    let updated = no_change(apply_edit_batch(
        &project,
        &batch("op_authorship_noop", &project, vec![operation]),
    ));
    assert_eq!(updated.project.materials[0], material);
}

fn authored_project() -> ProjectEnvelope {
    let mut project = sample_project();
    project.project.authorship = Some(ProjectAuthorship {
        entity: entity(&["project"]),
        multicam_groups: Vec::new(),
        annotations: Vec::new(),
        deliveries: vec![DeliveryAuthorship {
            render_config_id: project.project.render_configs[0].id.clone(),
            entity: entity(&["project", "delivery"]),
        }],
    });
    project.project.sequences[0].authorship = Some(SequenceAuthorship::Otio {
        document_sha256: Sha256Digest::new("a".repeat(64)),
    });
    project
}

fn batch(id: &str, project: &ProjectEnvelope, operations: Vec<EditOperation>) -> EditBatch {
    EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: Vec::new(),
        operations,
    }
}

fn applied(outcome: EditOutcome) -> ProjectEnvelope {
    match outcome {
        EditOutcome::Applied { project, .. } => project,
        other => panic!("expected applied edit, got {other:?}"),
    }
}

fn no_change(outcome: EditOutcome) -> ProjectEnvelope {
    match outcome {
        EditOutcome::NoChange { project, .. } => project,
        other => panic!("expected no-change edit, got {other:?}"),
    }
}

fn entity(path: &[&str]) -> EntityAuthorship {
    let definition = AuthoredDefinitionId::new("fn_main");
    let site = AuthoredSite {
        definition: definition.clone(),
        function: AuthoredFunctionName::new("main"),
        source: AuthoredSourceId::new("main.veac"),
        span: AuthoredSpan { start: 0, end: 4 },
    };
    EntityAuthorship {
        logical_path: path
            .iter()
            .map(|segment| LogicalPathSegment::new(*segment))
            .collect(),
        events: vec![AuthorshipEvent {
            kind: AuthorshipEventKind::Constructor,
            operation: DomainOpcode(0x1001),
            origin: site,
            definition: AuthoredDefinition {
                identity: definition,
                kind: AuthoredDefinitionKind::Function,
                name: AuthoredFunctionName::new("main"),
                source: AuthoredSourceId::new("main.veac"),
                span: AuthoredSpan { start: 0, end: 4 },
            },
            call_stack: Vec::new(),
            iterations: Vec::new(),
        }],
    }
}
