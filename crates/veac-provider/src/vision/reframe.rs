use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{Rational, RationalTime};

use crate::validation::{increasing, probability, time};
use crate::{CropSample, InputArtifact, MediaType, ProviderResult, ReviewDecision, Validate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AutoReframeRequest {
    pub video: InputArtifact,
    pub target_aspect_ratio: Rational,
    pub subject_tracks: Vec<InputArtifact>,
    pub smoothing: f64,
    pub dead_zone: f64,
    pub maximum_speed_per_second: f64,
    pub manual_overrides: Vec<CropSample>,
}

impl Validate for AutoReframeRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video || !self.target_aspect_ratio.is_positive() {
            return crate::validation::invalid("auto reframe requires video and a positive aspect");
        }
        for track in &self.subject_tracks {
            track.validate()?;
            if track.media_type != MediaType::Analysis {
                return crate::validation::invalid("auto reframe tracks must be analysis data");
            }
        }
        probability(self.smoothing, "reframe smoothing")?;
        probability(self.dead_zone, "reframe dead zone")?;
        crate::validation::positive(self.maximum_speed_per_second, "reframe maximum speed")?;
        for value in &self.manual_overrides {
            value.validate()?;
        }
        increasing(
            &self
                .manual_overrides
                .iter()
                .map(|value| value.time)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReframeMovement {
    pub start: RationalTime,
    pub end: RationalTime,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AutoReframeResult {
    pub crops: Vec<CropSample>,
    pub movements: Vec<ReframeMovement>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for AutoReframeResult {
    fn validate(&self) -> ProviderResult<()> {
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
        let mut starts = Vec::with_capacity(self.movements.len());
        for movement in &self.movements {
            time(movement.start)?;
            time(movement.end)?;
            crate::validation::text(&movement.reason, "reframe movement reason")?;
            if movement.start >= movement.end {
                return crate::validation::invalid("reframe movement must have positive duration");
            }
            starts.push(movement.start);
        }
        increasing(&starts)?;
        for value in &self.decisions {
            value.validate()?;
        }
        Ok(())
    }
}
