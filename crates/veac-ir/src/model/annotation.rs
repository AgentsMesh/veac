use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub id: AnnotationId,
    pub target: AnnotationTarget,
    pub span: AnnotationSpan,
    pub payload: AnnotationPayload,
    pub provenance: Option<AnnotationProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnotationTarget {
    Project,
    Sequence { sequence_id: SequenceId },
    Track { track_id: TrackId },
    Clip { clip_id: ItemId },
    Material { material_id: MaterialId },
    MulticamGroup { group_id: MulticamGroupId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnotationSpan {
    Untimed,
    Point { at: RationalTime },
    Range { range: TimeRange },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnnotationProvenance {
    pub producer: String,
    pub request_sha256: String,
    pub response_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnotationPayload {
    Marker {
        label: String,
        color: Option<Color>,
    },
    Language {
        scores: Vec<LanguageConfidence>,
    },
    SceneBoundary {
        confidence: f64,
        hard_cut: bool,
    },
    Scene,
    Beat {
        confidence: f64,
        bar: u64,
        beat_in_bar: u16,
        tempo: Rational,
        meter: u16,
    },
    Silence {
        mean_db: f64,
        confidence: f64,
    },
    Filler {
        token: String,
        confidence: f64,
        suggestion: FillerSuggestion,
    },
    Highlight {
        score: f64,
        rationale: String,
        evidence: Vec<String>,
    },
    Review {
        action: String,
        rationale: String,
        confidence: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LanguageConfidence {
    pub language: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FillerSuggestion {
    Keep,
    Delete,
    Tighten,
}
