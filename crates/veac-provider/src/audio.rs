use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ArtifactKind;
use veac_ir::TimeRange;

use crate::validation::{finite, non_overlapping, probability, text};
use crate::{InputArtifact, MediaType, ProviderArtifact, ProviderResult, ScalarSample, Validate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DenoiseRequest {
    pub audio: InputArtifact,
    pub strength: f64,
    pub preserve_voice: bool,
    pub noise_only_ranges: Vec<TimeRange>,
}

impl Validate for DenoiseRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.media_type != MediaType::Audio {
            return crate::validation::invalid("denoise input must be audio");
        }
        probability(self.strength, "denoise strength")?;
        non_overlapping(&self.noise_only_ranges)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DenoiseResult {
    pub audio: ProviderArtifact,
    pub noise_reduction_db: Vec<ScalarSample>,
    pub residual_noise_db: f64,
}

impl Validate for DenoiseResult {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if !matches!(
            self.audio.kind(),
            ArtifactKind::AudioStem | ArtifactKind::ProxyAudio
        ) {
            return crate::validation::invalid("denoise output must be an audio artifact");
        }
        for sample in &self.noise_reduction_db {
            sample.validate()?;
        }
        crate::validation::increasing(
            &self
                .noise_reduction_db
                .iter()
                .map(|sample| sample.time)
                .collect::<Vec<_>>(),
        )?;
        finite(self.residual_noise_db, "residual noise")
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum StemKind {
    Vocals,
    Music,
    Dialogue,
    Effects,
    Ambience,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VocalSeparationRequest {
    pub audio: InputArtifact,
    pub stems: Vec<StemKind>,
    pub preserve_phase: bool,
}

impl Validate for VocalSeparationRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.media_type != MediaType::Audio {
            return crate::validation::invalid("source separation input must be audio");
        }
        if self.stems.is_empty() || !self.stems.windows(2).all(|pair| pair[0] < pair[1]) {
            return crate::validation::invalid("requested stems must be unique and sorted");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SeparatedStem {
    pub kind: StemKind,
    pub label: String,
    pub audio: ProviderArtifact,
    pub leakage: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VocalSeparationResult {
    pub stems: Vec<SeparatedStem>,
}

impl Validate for VocalSeparationResult {
    fn validate(&self) -> ProviderResult<()> {
        let mut previous = None;
        for stem in &self.stems {
            if previous.is_some_and(|value| value >= stem.kind) {
                return crate::validation::invalid("separated stems must be unique and sorted");
            }
            text(&stem.label, "stem label")?;
            stem.audio.validate()?;
            if stem.audio.kind() != ArtifactKind::AudioStem {
                return crate::validation::invalid(
                    "separation output must use audio-stem artifacts",
                );
            }
            probability(stem.leakage, "stem leakage")?;
            previous = Some(stem.kind);
        }
        if self.stems.is_empty() {
            crate::validation::invalid("source separation must return at least one stem")
        } else {
            Ok(())
        }
    }
}
