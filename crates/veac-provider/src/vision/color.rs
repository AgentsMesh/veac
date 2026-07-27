use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{Color, TimeRange};

use crate::validation::{finite, non_overlapping, probability};
use crate::{InputArtifact, MediaType, ProviderResult, ReviewDecision, Validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColorMatchIntent {
    Neutral,
    ReferenceLook,
    SkinTone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorMatchRequest {
    pub source: InputArtifact,
    pub reference: InputArtifact,
    pub source_samples: Vec<TimeRange>,
    pub reference_samples: Vec<TimeRange>,
    pub intent: ColorMatchIntent,
    pub protected_colors: Vec<Color>,
}

impl Validate for ColorMatchRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.source.validate()?;
        self.reference.validate()?;
        if !matches!(self.source.media_type, MediaType::Video | MediaType::Image)
            || !matches!(
                self.reference.media_type,
                MediaType::Video | MediaType::Image
            )
        {
            return crate::validation::invalid("color match inputs must be visual media");
        }
        non_overlapping(&self.source_samples)?;
        non_overlapping(&self.reference_samples)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorAdjustment {
    pub matrix: [f64; 9],
    pub offset: [f64; 3],
    pub exposure_stops: f64,
    pub temperature_kelvin: f64,
    pub tint: f64,
    pub contrast: f64,
    pub saturation: f64,
}

impl Validate for ColorAdjustment {
    fn validate(&self) -> ProviderResult<()> {
        for value in self.matrix.iter().chain(self.offset.iter()).chain([
            &self.exposure_stops,
            &self.temperature_kelvin,
            &self.tint,
            &self.contrast,
            &self.saturation,
        ]) {
            finite(*value, "color adjustment")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorMatchResult {
    pub adjustment: ColorAdjustment,
    pub confidence: f64,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for ColorMatchResult {
    fn validate(&self) -> ProviderResult<()> {
        self.adjustment.validate()?;
        probability(self.confidence, "color match confidence")?;
        for value in &self.decisions {
            value.validate()?;
        }
        Ok(())
    }
}
