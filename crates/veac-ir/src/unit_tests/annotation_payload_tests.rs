use crate::{test_support::sample_project, *};

#[path = "annotation_payload_tests/support.rs"]
mod support;
use support::*;

#[test]
fn every_analysis_and_review_payload_validates_on_its_time_domain() {
    let mut project = sample_project();
    project.project.annotations = vec![
        annotation(
            "ann_boundary",
            sequence_target(),
            point(100),
            AnnotationPayload::SceneBoundary {
                confidence: 0.9,
                hard_cut: true,
            },
        ),
        annotation(
            "ann_filler",
            clip_target(),
            range(100, 100),
            AnnotationPayload::Filler {
                token: "um".into(),
                confidence: 0.8,
                suggestion: FillerSuggestion::Tighten,
            },
        ),
        annotation(
            "ann_highlight",
            track_target(),
            range(0, 200),
            AnnotationPayload::Highlight {
                score: 0.95,
                rationale: "Clear product value".into(),
                evidence: vec!["benefit".into(), "demo".into()],
            },
        ),
        annotation(
            "ann_language",
            AnnotationTarget::Material {
                material_id: MaterialId::new("med_video").unwrap(),
            },
            AnnotationSpan::Untimed,
            AnnotationPayload::Language {
                scores: vec![
                    language("en-US", 0.8),
                    language("zh-Hans", 0.8),
                    language("fr", 0.5),
                ],
            },
        ),
        annotation(
            "ann_review",
            sequence_target(),
            range(200, 100),
            AnnotationPayload::Review {
                action: "shorten".into(),
                rationale: "Repeated point".into(),
                confidence: 0.75,
            },
        ),
        annotation(
            "ann_scene",
            sequence_target(),
            range(0, 300),
            AnnotationPayload::Scene,
        ),
        annotation(
            "ann_silence",
            track_target(),
            range(300, 100),
            AnnotationPayload::Silence {
                mean_db: -58.0,
                confidence: 1.0,
            },
        ),
    ];
    validate(&project).unwrap();
    assert_eq!(
        decode_canonical_json(&canonical_json(&project).unwrap()).unwrap(),
        project
    );
}

#[test]
fn payload_contracts_reject_wrong_spans_text_scores_and_ordering() {
    let cases = vec![
        annotation(
            "ann_boundary",
            sequence_target(),
            range(0, 100),
            AnnotationPayload::SceneBoundary {
                confidence: f64::NAN,
                hard_cut: false,
            },
        ),
        annotation(
            "ann_scene",
            sequence_target(),
            point(0),
            AnnotationPayload::Scene,
        ),
        annotation(
            "ann_silence",
            sequence_target(),
            range(0, 100),
            AnnotationPayload::Silence {
                mean_db: f64::INFINITY,
                confidence: 0.5,
            },
        ),
        annotation(
            "ann_filler",
            clip_target(),
            range(0, 100),
            AnnotationPayload::Filler {
                token: " \0".into(),
                confidence: 0.5,
                suggestion: FillerSuggestion::Delete,
            },
        ),
        annotation(
            "ann_highlight",
            sequence_target(),
            range(0, 100),
            AnnotationPayload::Highlight {
                score: 1.1,
                rationale: "reason".into(),
                evidence: vec!["z".into(), "a".into()],
            },
        ),
        annotation(
            "ann_review",
            sequence_target(),
            range(0, 100),
            AnnotationPayload::Review {
                action: " ".into(),
                rationale: "reason".into(),
                confidence: -0.1,
            },
        ),
        annotation(
            "ann_language",
            AnnotationTarget::Material {
                material_id: MaterialId::new("med_video").unwrap(),
            },
            AnnotationSpan::Untimed,
            AnnotationPayload::Language {
                scores: vec![language("fr", 0.4), language("bad_tag!", 0.9)],
            },
        ),
    ];
    for value in cases {
        let mut project = sample_project();
        project.project.annotations = vec![value];
        assert!(validate(&project)
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|value| value.code == "ANNOTATION_PAYLOAD"));
    }
}

#[test]
fn highlight_evidence_preserves_authored_semantic_order() {
    let mut project = sample_project();
    project.project.annotations = vec![annotation(
        "ann_highlight",
        sequence_target(),
        range(0, 100),
        AnnotationPayload::Highlight {
            score: 0.9,
            rationale: "ordered evidence".into(),
            evidence: vec!["second observation".into(), "first observation".into()],
        },
    )];
    validate(&project).unwrap();
    let json = canonical_json(&project).unwrap();
    assert_eq!(decode_canonical_json(&json).unwrap(), project);
}
