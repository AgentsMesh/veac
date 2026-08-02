use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{RationalTime, TimeRange};

use crate::validation::{non_overlapping, probability, text, time};
use crate::{InputArtifact, MediaType, ProviderResult, ReviewDecision, Validate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FillerDetectionRequest {
    pub transcript: InputArtifact,
    pub language: String,
    pub filler_lexicon: Vec<String>,
    pub context_padding: RationalTime,
}

impl Validate for FillerDetectionRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.transcript.validate()?;
        if self.transcript.media_type != MediaType::Text {
            return crate::validation::invalid("filler detection input must be a transcript");
        }
        crate::validation::language(&self.language)?;
        if !self.filler_lexicon.windows(2).all(|pair| pair[0] < pair[1]) {
            return crate::validation::invalid("filler lexicon must be unique and sorted");
        }
        for filler in &self.filler_lexicon {
            text(filler, "filler token")?;
        }
        time(self.context_padding)?;
        if self.context_padding.value < 0 {
            crate::validation::invalid("filler context padding cannot be negative")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FillerAction {
    Keep,
    Delete,
    Tighten,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FillerOccurrence {
    pub range: TimeRange,
    pub token: String,
    pub confidence: f64,
    pub suggested_action: FillerAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FillerDetectionResult {
    pub occurrences: Vec<FillerOccurrence>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for FillerDetectionResult {
    fn validate(&self) -> ProviderResult<()> {
        for occurrence in &self.occurrences {
            crate::validation::range(occurrence.range)?;
            text(&occurrence.token, "filler occurrence")?;
            probability(occurrence.confidence, "filler confidence")?;
        }
        non_overlapping(
            &self
                .occurrences
                .iter()
                .map(|value| value.range)
                .collect::<Vec<_>>(),
        )?;
        for decision in &self.decisions {
            decision.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HighlightDetectionRequest {
    pub inputs: Vec<InputArtifact>,
    pub objective: String,
    pub target_duration: Option<RationalTime>,
    pub maximum_highlights: u32,
}

impl Validate for HighlightDetectionRequest {
    fn validate(&self) -> ProviderResult<()> {
        if self.inputs.is_empty() || self.maximum_highlights == 0 {
            return crate::validation::invalid(
                "highlight analysis requires input and output count",
            );
        }
        for input in &self.inputs {
            input.validate()?;
        }
        text(&self.objective, "highlight objective")?;
        if let Some(value) = self.target_duration {
            time(value)?;
            if value.value <= 0 {
                return crate::validation::invalid("highlight target duration must be positive");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Highlight {
    pub range: TimeRange,
    pub score: f64,
    pub rationale: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HighlightDetectionResult {
    pub highlights: Vec<Highlight>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for HighlightDetectionResult {
    fn validate(&self) -> ProviderResult<()> {
        for highlight in &self.highlights {
            crate::validation::range(highlight.range)?;
            probability(highlight.score, "highlight score")?;
            text(&highlight.rationale, "highlight rationale")?;
            if !highlight.evidence.windows(2).all(|pair| pair[0] < pair[1]) {
                return crate::validation::invalid("highlight evidence must be unique and sorted");
            }
        }
        non_overlapping(
            &self
                .highlights
                .iter()
                .map(|value| value.range)
                .collect::<Vec<_>>(),
        )?;
        for decision in &self.decisions {
            decision.validate()?;
        }
        Ok(())
    }
}
