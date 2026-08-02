use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ArtifactKind;
use veac_ir::{Rect, TimeRange};

use crate::validation::{non_overlapping, normalized_rect, probability, text};
use crate::{
    InputArtifact, MediaType, ProviderArtifact, ProviderResult, ReviewDecision, ScalarSample,
    Validate,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubjectPrompt {
    pub id: String,
    pub label: String,
    pub initial_rect: Option<Rect>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SegmentationRequest {
    pub video: InputArtifact,
    pub subjects: Vec<SubjectPrompt>,
    pub temporal_consistency: f64,
    pub edge_refinement: f64,
    pub correction_mattes: Vec<InputArtifact>,
}

impl Validate for SegmentationRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video || self.subjects.is_empty() {
            return crate::validation::invalid("segmentation requires video and subjects");
        }
        let mut previous: Option<&str> = None;
        for subject in &self.subjects {
            text(&subject.id, "subject id")?;
            text(&subject.label, "subject label")?;
            if previous.is_some_and(|value| value >= subject.id.as_str()) {
                return crate::validation::invalid(
                    "segmentation subjects must be unique and sorted",
                );
            }
            if let Some(value) = subject.initial_rect {
                normalized_rect(value)?;
            }
            previous = Some(&subject.id);
        }
        probability(self.temporal_consistency, "temporal consistency")?;
        probability(self.edge_refinement, "edge refinement")?;
        for matte in &self.correction_mattes {
            matte.validate()?;
            if matte.media_type != MediaType::Matte {
                return crate::validation::invalid("segmentation corrections must be mattes");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubjectVisibility {
    pub subject_id: String,
    pub visible_ranges: Vec<TimeRange>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SegmentationResult {
    pub matte: ProviderArtifact,
    pub subjects: Vec<SubjectVisibility>,
    pub quality: Vec<ScalarSample>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for SegmentationResult {
    fn validate(&self) -> ProviderResult<()> {
        validate_matte(&self.matte)?;
        let mut previous: Option<&str> = None;
        for subject in &self.subjects {
            text(&subject.subject_id, "segmented subject id")?;
            if previous.is_some_and(|value| value >= subject.subject_id.as_str()) {
                return crate::validation::invalid("segmented subjects must be unique and sorted");
            }
            non_overlapping(&subject.visible_ranges)?;
            probability(subject.confidence, "subject confidence")?;
            previous = Some(&subject.subject_id);
        }
        for value in &self.quality {
            value.validate()?;
        }
        crate::validation::increasing(
            &self
                .quality
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )?;
        for decision in &self.decisions {
            decision.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatteOperation {
    Union,
    Intersection,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatteRequest {
    pub mattes: Vec<InputArtifact>,
    pub operation: MatteOperation,
    pub feather_pixels: f64,
    pub expansion_pixels: f64,
}

impl Validate for MatteRequest {
    fn validate(&self) -> ProviderResult<()> {
        if self.mattes.is_empty() {
            return crate::validation::invalid("matte operation requires at least one input");
        }
        for matte in &self.mattes {
            matte.validate()?;
            if matte.media_type != MediaType::Matte {
                return crate::validation::invalid("matte operations only accept matte inputs");
            }
        }
        crate::validation::finite(self.feather_pixels, "matte feather")?;
        crate::validation::finite(self.expansion_pixels, "matte expansion")?;
        if self.feather_pixels < 0.0 {
            crate::validation::invalid("matte feather cannot be negative")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatteResult {
    pub matte: ProviderArtifact,
    pub coverage: Vec<ScalarSample>,
}

impl Validate for MatteResult {
    fn validate(&self) -> ProviderResult<()> {
        validate_matte(&self.matte)?;
        for value in &self.coverage {
            value.validate()?;
            probability(value.value, "matte coverage")?;
        }
        crate::validation::increasing(
            &self
                .coverage
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )
    }
}

fn validate_matte(value: &ProviderArtifact) -> ProviderResult<()> {
    value.validate()?;
    if value.kind() == ArtifactKind::Matte {
        Ok(())
    } else {
        crate::validation::invalid("segmentation output must be a matte artifact")
    }
}
