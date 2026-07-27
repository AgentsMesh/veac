use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{Rational, RationalTime, TimeRange};

use crate::validation::{increasing, non_overlapping, probability, time};
use crate::{InputArtifact, MediaType, ProviderResult, Validate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneDetectionRequest {
    pub video: InputArtifact,
    pub sensitivity: f64,
    pub minimum_scene_duration: RationalTime,
}

impl Validate for SceneDetectionRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video {
            return crate::validation::invalid("scene detection input must be video");
        }
        probability(self.sensitivity, "scene sensitivity")?;
        time(self.minimum_scene_duration)?;
        if self.minimum_scene_duration.value <= 0 {
            return crate::validation::invalid("minimum scene duration must be positive");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneBoundary {
    pub time: RationalTime,
    pub confidence: f64,
    pub hard_cut: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneDetectionResult {
    pub boundaries: Vec<SceneBoundary>,
    pub scenes: Vec<TimeRange>,
}

impl Validate for SceneDetectionResult {
    fn validate(&self) -> ProviderResult<()> {
        for boundary in &self.boundaries {
            time(boundary.time)?;
            probability(boundary.confidence, "scene confidence")?;
        }
        increasing(
            &self
                .boundaries
                .iter()
                .map(|boundary| boundary.time)
                .collect::<Vec<_>>(),
        )?;
        non_overlapping(&self.scenes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BeatDetectionRequest {
    pub audio: InputArtifact,
    pub tempo_range: Option<(f64, f64)>,
    pub meter_hint: Option<u16>,
}

impl Validate for BeatDetectionRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.media_type != MediaType::Audio {
            return crate::validation::invalid("beat detection input must be audio");
        }
        if self
            .tempo_range
            .is_some_and(|(low, high)| !low.is_finite() || low <= 0.0 || high <= low)
            || self.meter_hint == Some(0)
        {
            return crate::validation::invalid("beat hints must be positive and ordered");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Beat {
    pub time: RationalTime,
    pub confidence: f64,
    pub bar: u64,
    pub beat_in_bar: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BeatDetectionResult {
    pub tempo: Rational,
    pub meter: u16,
    pub beats: Vec<Beat>,
}

impl Validate for BeatDetectionResult {
    fn validate(&self) -> ProviderResult<()> {
        if !self.tempo.is_positive() || self.meter == 0 {
            return crate::validation::invalid("detected tempo and meter must be positive");
        }
        for beat in &self.beats {
            time(beat.time)?;
            probability(beat.confidence, "beat confidence")?;
            if beat.beat_in_bar == 0 || beat.beat_in_bar > self.meter {
                return crate::validation::invalid("beat index must fit the detected meter");
            }
        }
        increasing(&self.beats.iter().map(|beat| beat.time).collect::<Vec<_>>())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SilenceDetectionRequest {
    pub audio: InputArtifact,
    pub threshold_db: f64,
    pub minimum_duration: RationalTime,
}

impl Validate for SilenceDetectionRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        crate::validation::finite(self.threshold_db, "silence threshold")?;
        time(self.minimum_duration)?;
        if self.audio.media_type != MediaType::Audio || self.minimum_duration.value <= 0 {
            crate::validation::invalid("silence detection requires audio and positive duration")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetectedSilence {
    pub range: TimeRange,
    pub mean_db: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SilenceDetectionResult {
    pub intervals: Vec<DetectedSilence>,
}

impl Validate for SilenceDetectionResult {
    fn validate(&self) -> ProviderResult<()> {
        for value in &self.intervals {
            crate::validation::range(value.range)?;
            crate::validation::finite(value.mean_db, "silence mean level")?;
            probability(value.confidence, "silence confidence")?;
        }
        non_overlapping(
            &self
                .intervals
                .iter()
                .map(|value| value.range)
                .collect::<Vec<_>>(),
        )
    }
}
