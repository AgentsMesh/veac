use crate::test_support::time;
use crate::*;

pub(super) fn marker(id: &str, target: AnnotationTarget, span: AnnotationSpan) -> Annotation {
    Annotation {
        id: AnnotationId::new(id).unwrap(),
        target,
        span,
        payload: AnnotationPayload::Marker {
            label: "Marker".into(),
            color: Some(Color {
                red: 12,
                green: 34,
                blue: 56,
                alpha: 255,
            }),
        },
        provenance: None,
    }
}

pub(super) fn beat(id: &str, at: i64) -> Annotation {
    Annotation {
        id: AnnotationId::new(id).unwrap(),
        target: AnnotationTarget::Clip {
            clip_id: ItemId::new("itm_video").unwrap(),
        },
        span: AnnotationSpan::Point { at: time(at) },
        payload: AnnotationPayload::Beat {
            confidence: 0.9,
            bar: 1,
            beat_in_bar: 1,
            tempo: Rational::new(120, 1).unwrap(),
            meter: 4,
        },
        provenance: None,
    }
}

pub(super) fn language() -> Annotation {
    let mut value = marker(
        "ann_language",
        AnnotationTarget::Material {
            material_id: MaterialId::new("med_video").unwrap(),
        },
        AnnotationSpan::Untimed,
    );
    value.payload = AnnotationPayload::Language {
        scores: vec![LanguageConfidence {
            language: "en-US".into(),
            confidence: 0.99,
        }],
    };
    value.provenance = Some(AnnotationProvenance {
        producer: "provider/request".into(),
        request_sha256: "a".repeat(64),
        response_sha256: "b".repeat(64),
    });
    value
}

pub(super) fn assert_invalid(project: &ProjectEnvelope, annotations: Vec<Annotation>, code: &str) {
    let mut value = project.clone();
    value.project.annotations = annotations;
    let diagnostics = validate(&value).unwrap_err().into_diagnostics();
    assert!(diagnostics.iter().any(|value| value.code == code));
}

pub(super) fn apply(
    project: &ProjectEnvelope,
    id: &str,
    operations: Vec<EditOperation>,
) -> EditOutcome {
    apply_edit_batch(
        project,
        &EditBatch {
            operation_id: OperationId::new(id).unwrap(),
            base_revision: project.project.revision,
            atomic: true,
            preconditions: vec![],
            operations,
        },
    )
}

pub(super) fn applied(outcome: EditOutcome) -> (ProjectEnvelope, Vec<ChangedObjectId>) {
    match outcome {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied annotation edit, got {other:?}"),
    }
}
