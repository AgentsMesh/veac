use veac_ir::{Length, LengthUnit, Point, Vec2};

use super::*;

pub(crate) fn word() -> TranscriptWord {
    TranscriptWord {
        range: range(0, 10),
        text: "hello".into(),
        confidence: 0.9,
    }
}

pub(crate) fn transform() -> TransformSample {
    TransformSample {
        time: time(0),
        position: Point {
            x: Length {
                value: 0.1,
                unit: LengthUnit::Normalized,
            },
            y: Length {
                value: 0.2,
                unit: LengthUnit::Normalized,
            },
        },
        scale: Vec2 { x: 1.0, y: 1.0 },
        rotation_degrees: 0.0,
        confidence: 0.9,
    }
}

pub(crate) fn filler_output() -> ProviderOutput {
    ProviderOutput::FillerDetection(FillerDetectionResult {
        occurrences: vec![FillerOccurrence {
            range: range(0, 5),
            token: "um".into(),
            confidence: 0.8,
            suggested_action: FillerAction::Delete,
        }],
        decisions: vec![decision()],
    })
}

pub(crate) fn highlight_output() -> ProviderOutput {
    ProviderOutput::HighlightDetection(HighlightDetectionResult {
        highlights: vec![Highlight {
            range: range(0, 10),
            score: 0.9,
            rationale: "clear statement".into(),
            evidence: vec!["speech".into()],
        }],
        decisions: vec![decision()],
    })
}

pub(crate) fn reframe_output() -> ProviderOutput {
    ProviderOutput::AutoReframe(AutoReframeResult {
        crops: vec![crop()],
        movements: vec![ReframeMovement {
            start: time(0),
            end: time(10),
            reason: "subject moved".into(),
        }],
        decisions: vec![decision()],
    })
}
