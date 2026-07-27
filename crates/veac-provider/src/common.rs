use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;
use veac_ir::{Point, RationalTime, Rect, TimeRange, Vec2};

use crate::validation::{finite, normalized_rect, probability, range, text, time};
use crate::{ProviderResult, Validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Text,
    Matte,
    Analysis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputArtifact {
    pub content: ContentDigest,
    pub media_type: MediaType,
    pub stream_index: Option<u32>,
    pub range: Option<TimeRange>,
}

impl Validate for InputArtifact {
    fn validate(&self) -> ProviderResult<()> {
        self.content.validate()?;
        if let Some(value) = self.range {
            range(value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceLabel {
    pub label: String,
    pub confidence: f64,
}

impl Validate for ConfidenceLabel {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.label, "label")?;
        probability(self.confidence, "confidence")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScalarSample {
    pub time: RationalTime,
    pub value: f64,
    pub confidence: Option<f64>,
}

impl Validate for ScalarSample {
    fn validate(&self) -> ProviderResult<()> {
        time(self.time)?;
        finite(self.value, "sample value")?;
        if let Some(value) = self.confidence {
            probability(value, "sample confidence")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransformSample {
    pub time: RationalTime,
    /// Absolute clip transform position with an explicit unit on each axis.
    pub position: Point,
    pub scale: Vec2,
    pub rotation_degrees: f64,
    pub confidence: f64,
}

impl Validate for TransformSample {
    fn validate(&self) -> ProviderResult<()> {
        time(self.time)?;
        for value in [
            self.position.x.value,
            self.position.y.value,
            self.scale.x,
            self.scale.y,
            self.rotation_degrees,
        ] {
            finite(value, "transform component")?;
        }
        probability(self.confidence, "transform confidence")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropSample {
    pub time: RationalTime,
    pub rect: Rect,
    pub confidence: f64,
}

impl Validate for CropSample {
    fn validate(&self) -> ProviderResult<()> {
        time(self.time)?;
        normalized_rect(self.rect)?;
        probability(self.confidence, "crop confidence")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewDecision {
    pub id: String,
    pub range: TimeRange,
    pub action: String,
    pub rationale: String,
    pub confidence: f64,
}

impl Validate for ReviewDecision {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.id, "decision id")?;
        range(self.range)?;
        text(&self.action, "decision action")?;
        text(&self.rationale, "decision rationale")?;
        probability(self.confidence, "decision confidence")
    }
}
