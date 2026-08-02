use veac_ir::*;

use crate::OtioImportResult;

use super::support::{project, time};

pub(super) fn annotated_project() -> ProjectEnvelope {
    let mut value = project();
    value.project.annotations = vec![
        Annotation {
            id: AnnotationId::new("ann_clip_beat").unwrap(),
            target: AnnotationTarget::Clip {
                clip_id: ItemId::new("itm_clip").unwrap(),
            },
            span: AnnotationSpan::Point { at: time(100) },
            payload: AnnotationPayload::Beat {
                confidence: 0.9,
                bar: 1,
                beat_in_bar: 1,
                tempo: Rational::new(120, 1).unwrap(),
                meter: 4,
            },
            provenance: Some(AnnotationProvenance {
                producer: "provider/beat".into(),
                request_sha256: "a".repeat(64),
                response_sha256: "b".repeat(64),
            }),
        },
        marker(
            "ann_project",
            AnnotationTarget::Project,
            AnnotationSpan::Untimed,
        ),
        marker(
            "ann_sequence",
            AnnotationTarget::Sequence {
                sequence_id: SequenceId::new("seq_main").unwrap(),
            },
            AnnotationSpan::Point { at: time(300) },
        ),
    ];
    veac_ir::validate(&value).unwrap();
    value
}

pub(super) fn remap_imported(value: &mut OtioImportResult) {
    value.sequence.id = SequenceId::new("seq_imported_annotations").unwrap();
    value.sequence.tracks[0].id = TrackId::new("trk_imported_annotations").unwrap();
    value.sequence.tracks[0].clips[0].id = ItemId::new("itm_imported_annotations").unwrap();
    for annotation in &mut value.annotations {
        match &mut annotation.target {
            AnnotationTarget::Sequence { sequence_id } => {
                *sequence_id = value.sequence.id.clone();
            }
            AnnotationTarget::Track { track_id } => {
                *track_id = value.sequence.tracks[0].id.clone();
            }
            AnnotationTarget::Clip { clip_id } => {
                *clip_id = value.sequence.tracks[0].clips[0].id.clone();
            }
            _ => {}
        }
    }
}

pub(super) fn marker(id: &str, target: AnnotationTarget, span: AnnotationSpan) -> Annotation {
    Annotation {
        id: AnnotationId::new(id).unwrap(),
        target,
        span,
        payload: AnnotationPayload::Marker {
            label: id.into(),
            color: None,
        },
        provenance: None,
    }
}
