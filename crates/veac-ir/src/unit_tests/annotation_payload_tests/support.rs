use crate::*;

pub(super) fn annotation(
    id: &str,
    target: AnnotationTarget,
    span: AnnotationSpan,
    payload: AnnotationPayload,
) -> Annotation {
    Annotation {
        id: AnnotationId::new(id).unwrap(),
        target,
        span,
        payload,
        provenance: None,
    }
}

pub(super) fn sequence_target() -> AnnotationTarget {
    AnnotationTarget::Sequence {
        sequence_id: SequenceId::new("seq_main").unwrap(),
    }
}

pub(super) fn track_target() -> AnnotationTarget {
    AnnotationTarget::Track {
        track_id: TrackId::new("trk_video").unwrap(),
    }
}

pub(super) fn clip_target() -> AnnotationTarget {
    AnnotationTarget::Clip {
        clip_id: ItemId::new("itm_video").unwrap(),
    }
}

pub(super) fn point(value: i64) -> AnnotationSpan {
    AnnotationSpan::Point {
        at: RationalTime::new(value, 600).unwrap(),
    }
}

pub(super) fn range(start: i64, duration: i64) -> AnnotationSpan {
    AnnotationSpan::Range {
        range: TimeRange::new(
            RationalTime::new(start, 600).unwrap(),
            RationalTime::new(duration, 600).unwrap(),
        )
        .unwrap(),
    }
}

pub(super) fn language(language: &str, confidence: f64) -> LanguageConfidence {
    LanguageConfidence {
        language: language.into(),
        confidence,
    }
}
