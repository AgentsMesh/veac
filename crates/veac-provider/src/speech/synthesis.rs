use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ArtifactKind;
use veac_ir::{Rational, TimeRange};

use crate::validation::{language, non_overlapping, text};
use crate::{ProviderArtifact, ProviderResult, TranscriptWord, Validate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VoiceSpec {
    pub voice: String,
    pub language: String,
    pub speaking_rate: Rational,
    pub pitch_semitones: f64,
}

impl Validate for VoiceSpec {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.voice, "voice id")?;
        language(&self.language)?;
        if !self.speaking_rate.is_positive() {
            return crate::validation::invalid("speaking rate must be a positive ratio");
        }
        crate::validation::finite(self.pitch_semitones, "voice pitch")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TtsRequest {
    pub text: String,
    pub voice: VoiceSpec,
    pub sample_rate: u32,
    pub target_range: Option<TimeRange>,
}

impl Validate for TtsRequest {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.text, "speech text")?;
        self.voice.validate()?;
        if self.sample_rate == 0 {
            return crate::validation::invalid("speech sample rate must be positive");
        }
        if let Some(value) = self.target_range {
            crate::validation::range(value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpeechResult {
    pub audio: ProviderArtifact,
    pub range: TimeRange,
    pub words: Vec<TranscriptWord>,
}

impl Validate for SpeechResult {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.kind() != ArtifactKind::Speech {
            return crate::validation::invalid("speech output must use a speech artifact");
        }
        crate::validation::range(self.range)?;
        for word in &self.words {
            word.validate()?;
        }
        non_overlapping(&self.words.iter().map(|word| word.range).collect::<Vec<_>>())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DubbingTurn {
    pub id: String,
    pub source_range: TimeRange,
    pub source_text: String,
    pub translated_text: String,
    pub voice: VoiceSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DubbingRequest {
    pub audio: crate::InputArtifact,
    pub source_language: String,
    pub target_language: String,
    pub turns: Vec<DubbingTurn>,
    pub preserve_timing: bool,
}

impl Validate for DubbingRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.media_type != crate::MediaType::Audio {
            return crate::validation::invalid("dubbing input must be audio");
        }
        language(&self.source_language)?;
        language(&self.target_language)?;
        let mut previous: Option<&str> = None;
        for turn in &self.turns {
            text(&turn.id, "dubbing turn id")?;
            crate::validation::range(turn.source_range)?;
            text(&turn.source_text, "dubbing source text")?;
            text(&turn.translated_text, "dubbing translated text")?;
            turn.voice.validate()?;
            if previous.is_some_and(|value| value >= turn.id.as_str()) {
                return crate::validation::invalid("dubbing turns must be unique and sorted");
            }
            previous = Some(&turn.id);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DubbedTurn {
    pub id: String,
    pub source_range: TimeRange,
    pub rendered_range: TimeRange,
    pub timing_scale: Rational,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DubbingResult {
    pub audio: ProviderArtifact,
    pub turns: Vec<DubbedTurn>,
}

impl Validate for DubbingResult {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.kind() != ArtifactKind::Speech {
            return crate::validation::invalid("dubbing output must use a speech artifact");
        }
        let mut previous: Option<&str> = None;
        for turn in &self.turns {
            text(&turn.id, "dubbed turn id")?;
            crate::validation::range(turn.source_range)?;
            crate::validation::range(turn.rendered_range)?;
            if !turn.timing_scale.is_positive()
                || previous.is_some_and(|value| value >= turn.id.as_str())
            {
                return crate::validation::invalid("dubbed turns require sorted ids and timing");
            }
            previous = Some(&turn.id);
        }
        Ok(())
    }
}
