use crate::test_support::{sample_project, time};
use crate::*;

#[path = "annotation_tests/support.rs"]
mod support;
use support::*;

#[test]
fn typed_annotations_validate_and_round_trip_canonically() {
    let mut project = sample_project();
    project.project.annotations = vec![
        marker(
            "ann_intro",
            AnnotationTarget::Project,
            AnnotationSpan::Untimed,
        ),
        language(),
        beat("ann_beat", 30),
    ];
    project.project.annotations.sort_by(|a, b| a.id.cmp(&b.id));
    validate(&project).unwrap();
    let json = canonical_json(&project).unwrap();
    assert_eq!(decode_canonical_json(&json).unwrap(), project);
    assert!(json.contains("provider/request"));
}

#[test]
fn annotation_validation_rejects_order_targets_time_payload_bounds_and_provenance() {
    let project = sample_project();
    assert_invalid(
        &project,
        vec![marker(
            "ann_bad_target",
            AnnotationTarget::Clip {
                clip_id: ItemId::new("itm_missing").unwrap(),
            },
            AnnotationSpan::Untimed,
        )],
        "ANNOTATION_TARGET_NOT_FOUND",
    );
    assert_invalid(
        &project,
        vec![marker(
            "ann_timed_project",
            AnnotationTarget::Project,
            AnnotationSpan::Point { at: time(1) },
        )],
        "ANNOTATION_TIME_DOMAIN",
    );
    assert_invalid(
        &project,
        vec![beat("ann_bounds", 601)],
        "ANNOTATION_CLIP_BOUNDS",
    );

    let mut bad_payload = beat("ann_payload", 1);
    let AnnotationPayload::Beat { confidence, .. } = &mut bad_payload.payload else {
        unreachable!()
    };
    *confidence = 1.1;
    assert_invalid(&project, vec![bad_payload], "ANNOTATION_PAYLOAD");

    let mut bad_provenance = marker(
        "ann_provenance",
        AnnotationTarget::Project,
        AnnotationSpan::Untimed,
    );
    bad_provenance.provenance = Some(AnnotationProvenance {
        producer: "provider/model".into(),
        request_sha256: "bad".into(),
        response_sha256: "b".repeat(64),
    });
    assert_invalid(&project, vec![bad_provenance], "ANNOTATION_PROVENANCE");

    let values = vec![beat("ann_z", 1), beat("ann_a", 2)];
    assert_invalid(&project, values, "ANNOTATION_ORDER");
    let duplicate = vec![beat("ann_same", 1), beat("ann_same", 2)];
    assert_invalid(&project, duplicate, "DUPLICATE_ANNOTATION_ID");
}

#[test]
fn annotation_edits_insert_sort_set_remove_and_rollback_atomically() {
    let project = sample_project();
    let inserted = apply(
        &project,
        "op_annotation_insert",
        vec![
            EditOperation::InsertAnnotation {
                annotation: Box::new(beat("ann_z", 20)),
            },
            EditOperation::InsertAnnotation {
                annotation: Box::new(beat("ann_a", 10)),
            },
        ],
    );
    let (mut project, changed) = applied(inserted);
    assert_eq!(project.project.annotations[0].id.as_str(), "ann_a");
    assert!(changed.contains(&ChangedObjectId::Annotation {
        id: AnnotationId::new("ann_z").unwrap(),
    }));

    let mut replacement = project.project.annotations[0].clone();
    replacement.payload = AnnotationPayload::Marker {
        label: "Reviewed beat".into(),
        color: None,
    };
    replacement.span = AnnotationSpan::Point { at: time(10) };
    project = applied(apply(
        &project,
        "op_annotation_set",
        vec![EditOperation::SetAnnotation {
            annotation: Box::new(replacement.clone()),
        }],
    ))
    .0;
    assert_eq!(project.project.annotations[0], replacement);
    project = applied(apply(
        &project,
        "op_annotation_remove",
        vec![EditOperation::RemoveAnnotation {
            annotation_id: AnnotationId::new("ann_z").unwrap(),
        }],
    ))
    .0;
    assert_eq!(project.project.annotations.len(), 1);

    let before = project.clone();
    let rejected = apply(
        &project,
        "op_annotation_atomic",
        vec![
            EditOperation::InsertAnnotation {
                annotation: Box::new(beat("ann_new", 30)),
            },
            EditOperation::InsertAnnotation {
                annotation: Box::new(replacement),
            },
        ],
    );
    assert!(matches!(rejected, EditOutcome::Rejected { .. }));
    assert_eq!(before, project);
}

#[test]
fn annotation_set_and_remove_distinguish_missing_ids_from_idempotence() {
    let project = sample_project();
    for (id, operation) in [
        (
            "op_annotation_set_missing",
            EditOperation::SetAnnotation {
                annotation: Box::new(beat("ann_missing", 10)),
            },
        ),
        (
            "op_annotation_remove_missing",
            EditOperation::RemoveAnnotation {
                annotation_id: AnnotationId::new("ann_missing").unwrap(),
            },
        ),
    ] {
        let outcome = apply(&project, id, vec![operation]);
        let EditOutcome::Rejected {
            current_revision,
            diagnostics,
        } = outcome
        else {
            panic!("missing annotation edit must reject")
        };
        assert_eq!(current_revision, project.project.revision);
        assert!(diagnostics[0].message.contains("annotation does not exist"));
    }

    let existing = beat("ann_existing", 10);
    let mut project = project;
    project.project.annotations = vec![existing.clone()];
    assert!(matches!(
        apply(
            &project,
            "op_annotation_set_same",
            vec![EditOperation::SetAnnotation {
                annotation: Box::new(existing),
            }],
        ),
        EditOutcome::NoChange { .. }
    ));
}
