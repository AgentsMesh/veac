use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ArtifactKind;
use veac_ir::{RationalTime, Rect, TimeRange};

use crate::validation::{increasing, non_overlapping, normalized_rect, probability, text, time};
use crate::{
    CropSample, InputArtifact, MediaType, ProviderArtifact, ProviderResult, ReviewDecision,
    TransformSample, Validate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrackingMode {
    Point,
    Object,
    Camera,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrackingTarget {
    Point { x: f64, y: f64 },
    Region { rect: Rect },
    Semantic { label: String, initial_rect: Rect },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrackingRequest {
    pub video: InputArtifact,
    pub mode: TrackingMode,
    pub target: TrackingTarget,
    pub sample_interval: RationalTime,
    pub smoothing: f64,
}

impl Validate for TrackingRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video {
            return crate::validation::invalid("tracking input must be video");
        }
        match &self.target {
            TrackingTarget::Point { x, y } => {
                probability(*x, "tracking point x")?;
                probability(*y, "tracking point y")?;
            }
            TrackingTarget::Region { rect } => normalized_rect(*rect)?,
            TrackingTarget::Semantic {
                label,
                initial_rect,
            } => {
                text(label, "tracking label")?;
                normalized_rect(*initial_rect)?;
            }
        }
        time(self.sample_interval)?;
        if self.sample_interval.value <= 0 {
            return crate::validation::invalid("tracking sample interval must be positive");
        }
        probability(self.smoothing, "tracking smoothing")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrackedRegion {
    pub time: RationalTime,
    pub rect: Rect,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrackingResult {
    pub track: ProviderArtifact,
    pub transforms: Vec<TransformSample>,
    pub regions: Vec<TrackedRegion>,
    pub lost_ranges: Vec<TimeRange>,
}

impl Validate for TrackingResult {
    fn validate(&self) -> ProviderResult<()> {
        self.track.validate()?;
        if self.track.kind() != ArtifactKind::MotionTrack {
            return crate::validation::invalid("tracking output must be a motion-track artifact");
        }
        for sample in &self.transforms {
            sample.validate()?;
        }
        increasing(
            &self
                .transforms
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )?;
        for region in &self.regions {
            time(region.time)?;
            normalized_rect(region.rect)?;
            probability(region.confidence, "tracked region confidence")?;
        }
        increasing(
            &self
                .regions
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )?;
        non_overlapping(&self.lost_ranges)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StabilizationRequest {
    pub video: InputArtifact,
    pub motion_track: Option<InputArtifact>,
    pub strength: f64,
    pub crop_margin: f64,
}

impl Validate for StabilizationRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video {
            return crate::validation::invalid("stabilization input must be video");
        }
        if let Some(track) = &self.motion_track {
            track.validate()?;
            if track.media_type != MediaType::Analysis {
                return crate::validation::invalid("stabilization track must be analysis data");
            }
        }
        probability(self.strength, "stabilization strength")?;
        probability(self.crop_margin, "stabilization crop margin")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StabilizationResult {
    pub transforms: Vec<TransformSample>,
    pub crops: Vec<CropSample>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for StabilizationResult {
    fn validate(&self) -> ProviderResult<()> {
        for value in &self.transforms {
            value.validate()?;
        }
        increasing(
            &self
                .transforms
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )?;
        for value in &self.crops {
            value.validate()?;
        }
        increasing(
            &self
                .crops
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )?;
        for value in &self.decisions {
            value.validate()?;
        }
        Ok(())
    }
}
