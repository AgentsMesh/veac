use super::*;

#[test]
fn language_and_scene_results_map_exact_annotations() {
    let language = analysis_case(Capability::LanguageDetection);
    assert_common(&language);
    assert_eq!(language.proposal.batch.operations.len(), 1);
    let value = annotation(&language.proposal, 0);
    assert_eq!(value.id.as_str(), "ann_provider_language_0001");
    assert_eq!(value.span, AnnotationSpan::Untimed);
    assert!(matches!(
        &value.payload,
        AnnotationPayload::Language { scores }
            if scores.len() == 1
                && scores[0].language == "en"
                && scores[0].confidence == 0.9
    ));
    assert_evidence(&language, 0, AnalysisEvidenceKind::Language, 0);

    let scene = analysis_case(Capability::SceneDetection);
    assert_common(&scene);
    assert_eq!(scene.proposal.batch.operations.len(), 2);
    let boundary = annotation(&scene.proposal, 0);
    assert_eq!(boundary.id.as_str(), "ann_provider_scene-boundary_0001");
    assert_eq!(boundary.span, AnnotationSpan::Point { at: time(10) });
    assert!(matches!(
        &boundary.payload,
        AnnotationPayload::SceneBoundary {
            confidence: 0.9,
            hard_cut: true
        }
    ));
    let range = TimeRange::new(time(0), time(10)).unwrap();
    let scene_value = annotation(&scene.proposal, 1);
    assert_eq!(scene_value.id.as_str(), "ann_provider_scene_0001");
    assert_eq!(scene_value.span, AnnotationSpan::Range { range });
    assert_eq!(scene_value.payload, AnnotationPayload::Scene);
    assert_evidence(&scene, 0, AnalysisEvidenceKind::SceneBoundary, 0);
    assert_evidence(&scene, 1, AnalysisEvidenceKind::Scene, 0);
}

#[test]
fn beat_and_silence_results_map_exact_annotations() {
    let beat = analysis_case(Capability::BeatDetection);
    assert_common(&beat);
    let value = annotation(&beat.proposal, 0);
    assert_eq!(value.id.as_str(), "ann_provider_beat_0001");
    assert_eq!(value.span, AnnotationSpan::Point { at: time(0) });
    assert!(matches!(
        &value.payload,
        AnnotationPayload::Beat {
            confidence: 0.9,
            bar: 1,
            beat_in_bar: 1,
            meter: 4,
            ..
        }
    ));
    assert_evidence(&beat, 0, AnalysisEvidenceKind::Beat, 0);

    let silence = analysis_case(Capability::SilenceDetection);
    assert_common(&silence);
    let value = annotation(&silence.proposal, 0);
    assert_eq!(value.id.as_str(), "ann_provider_silence_0001");
    assert_eq!(
        value.span,
        AnnotationSpan::Range {
            range: TimeRange::new(time(0), time(10)).unwrap()
        }
    );
    assert!(matches!(
        &value.payload,
        AnnotationPayload::Silence {
            mean_db: -60.0,
            confidence: 0.9
        }
    ));
    assert_evidence(&silence, 0, AnalysisEvidenceKind::Silence, 0);
}

#[test]
fn filler_and_highlight_results_map_annotations_and_review_decisions() {
    let filler = analysis_case(Capability::FillerDetection);
    assert_common(&filler);
    assert_eq!(filler.proposal.batch.operations.len(), 2);
    assert!(matches!(
        &annotation(&filler.proposal, 0).payload,
        AnnotationPayload::Filler {
            token,
            confidence: 0.8,
            suggestion: FillerSuggestion::Delete
        } if token == "um"
    ));
    assert!(matches!(
        &annotation(&filler.proposal, 1).payload,
        AnnotationPayload::Review {
            action,
            rationale,
            confidence: 0.9
        } if action == "review" && rationale == "provider evidence"
    ));
    assert_evidence(&filler, 0, AnalysisEvidenceKind::Filler, 0);
    assert_evidence(&filler, 1, AnalysisEvidenceKind::FillerDecision, 0);

    let highlight = analysis_case(Capability::HighlightDetection);
    assert_common(&highlight);
    assert_eq!(highlight.proposal.batch.operations.len(), 2);
    assert!(matches!(
        &annotation(&highlight.proposal, 0).payload,
        AnnotationPayload::Highlight {
            score: 0.9,
            rationale,
            evidence
        } if rationale == "clear statement" && evidence == &["speech"]
    ));
    assert!(matches!(
        &annotation(&highlight.proposal, 1).payload,
        AnnotationPayload::Review { action, rationale, .. }
            if action == "review" && rationale == "provider evidence"
    ));
    assert_evidence(&highlight, 0, AnalysisEvidenceKind::Highlight, 0);
    assert_evidence(&highlight, 1, AnalysisEvidenceKind::HighlightDecision, 0);
}
