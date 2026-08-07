use veac_ir::{AnnotationPayload, FillerSuggestion};

use super::super::support;
use super::SOURCE;

const MARKER: &str = "annotation_untimed(),\n        \
    annotation_marker(\"双机位访谈\", marker_color_present(#22aa66ff))";

#[test]
fn every_remaining_annotation_payload_variant_lowers() {
    let source = SOURCE.replace(
        "annotation_target_multicam(group)",
        "annotation_target_sequence(sequence)",
    );
    let cases = [
        (
            "annotation_untimed(), annotation_marker(\"无颜色\", marker_color_none())",
            "marker",
        ),
        (
            "annotation_point(1s), annotation_scene_boundary(80%, true)",
            "boundary",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_scene()",
            "scene",
        ),
        (
            "annotation_point(1s), annotation_beat(90%, 1, 2, 120.0, 4)",
            "beat",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_silence(-48.0, 75%)",
            "silence",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_filler(\"嗯\", 60%, filler_keep())",
            "keep",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_filler(\"啊\", 70%, filler_delete())",
            "delete",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_filler(\"呃\", 80%, filler_tighten())",
            "tighten",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_review(review_keep(), \"保留\", 90%)",
            "review_keep",
        ),
        (
            "annotation_range(during(0s, 1s)), annotation_review(review_remove(), \"删除\", 95%)",
            "review_remove",
        ),
    ];
    for (payload, expected) in cases {
        let envelope = support::envelope(&source.replace(MARKER, payload));
        let payload = &envelope.project.annotations[0].payload;
        match expected {
            "marker" => assert!(matches!(
                payload,
                AnnotationPayload::Marker { color: None, .. }
            )),
            "boundary" => assert!(matches!(payload, AnnotationPayload::SceneBoundary { .. })),
            "scene" => assert_eq!(payload, &AnnotationPayload::Scene),
            "beat" => assert!(matches!(payload, AnnotationPayload::Beat { .. })),
            "silence" => assert!(matches!(payload, AnnotationPayload::Silence { .. })),
            "keep" => assert_filler(payload, FillerSuggestion::Keep),
            "delete" => assert_filler(payload, FillerSuggestion::Delete),
            "tighten" => assert_filler(payload, FillerSuggestion::Tighten),
            "review_keep" => assert_review(payload, "keep"),
            "review_remove" => assert_review(payload, "remove"),
            _ => unreachable!(),
        }
    }
}

#[test]
fn annotation_numeric_and_language_conversion_failures_are_lowering_errors() {
    let source = SOURCE.replace(
        "annotation_target_multicam(group)",
        "annotation_target_sequence(sequence)",
    );
    for payload in [
        "annotation_beat(90%, -1, 2, 120.0, 4)",
        "annotation_beat(90%, 1, 70000, 120.0, 4)",
        "annotation_beat(90%, 1, 2, 120.0, 70000)",
    ] {
        let replacement = format!("annotation_point(1s), {payload}");
        let error = support::error(&source.replace(MARKER, &replacement));
        assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER");
    }
    let duplicate = SOURCE.replace(
        "language_confidence(\"en-US\", 90%)",
        "language_confidence(\"zh-Hans\", 90%)",
    );
    assert_eq!(support::error(&duplicate).code, "PROGRAM_EXECUTABLE_LOWER");
}

fn assert_filler(payload: &AnnotationPayload, expected: FillerSuggestion) {
    let AnnotationPayload::Filler { suggestion, .. } = payload else {
        panic!("expected filler payload")
    };
    assert_eq!(*suggestion, expected);
}

fn assert_review(payload: &AnnotationPayload, expected: &str) {
    let AnnotationPayload::Review { action, .. } = payload else {
        panic!("expected review payload")
    };
    assert_eq!(action, expected);
}
