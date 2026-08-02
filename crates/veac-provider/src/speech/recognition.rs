use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::TimeRange;

use crate::validation::{language, non_overlapping, probability, text};
use crate::{InputArtifact, MediaType, ProviderResult, Validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerMode {
    None,
    Detect,
    KnownCount { count: u16 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AsrRequest {
    pub audio: InputArtifact,
    pub language_hint: Option<String>,
    pub speakers: SpeakerMode,
    pub word_timing: bool,
}

impl Validate for AsrRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.audio.validate()?;
        if self.audio.media_type != MediaType::Audio {
            return crate::validation::invalid("ASR input must be audio");
        }
        if let Some(value) = &self.language_hint {
            language(value)?;
        }
        if matches!(self.speakers, SpeakerMode::KnownCount { count: 0 }) {
            return crate::validation::invalid("known speaker count must be positive");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranscriptWord {
    pub range: TimeRange,
    pub text: String,
    pub confidence: f64,
}

impl Validate for TranscriptWord {
    fn validate(&self) -> ProviderResult<()> {
        crate::validation::range(self.range)?;
        text(&self.text, "transcript word")?;
        probability(self.confidence, "word confidence")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranscriptSegment {
    pub id: String,
    pub range: TimeRange,
    pub speaker: Option<String>,
    pub text: String,
    pub confidence: f64,
    pub words: Vec<TranscriptWord>,
}

impl Validate for TranscriptSegment {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.id, "transcript segment id")?;
        crate::validation::range(self.range)?;
        if let Some(value) = &self.speaker {
            text(value, "speaker id")?;
        }
        text(&self.text, "transcript segment")?;
        probability(self.confidence, "segment confidence")?;
        for word in &self.words {
            word.validate()?;
        }
        non_overlapping(&self.words.iter().map(|word| word.range).collect::<Vec<_>>())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AsrResult {
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
}

impl Validate for AsrResult {
    fn validate(&self) -> ProviderResult<()> {
        language(&self.language)?;
        for segment in &self.segments {
            segment.validate()?;
        }
        non_overlapping(
            &self
                .segments
                .iter()
                .map(|segment| segment.range)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LanguageDetectionRequest {
    pub input: InputArtifact,
    pub candidates: Vec<String>,
}

impl Validate for LanguageDetectionRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.input.validate()?;
        for candidate in &self.candidates {
            language(candidate)?;
        }
        if !self.candidates.windows(2).all(|pair| pair[0] < pair[1]) {
            return crate::validation::invalid("language candidates must be unique and sorted");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LanguageScore {
    pub language: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LanguageDetectionResult {
    pub languages: Vec<LanguageScore>,
}

impl Validate for LanguageDetectionResult {
    fn validate(&self) -> ProviderResult<()> {
        let mut previous = f64::INFINITY;
        let mut previous_language: Option<&str> = None;
        for score in &self.languages {
            language(&score.language)?;
            probability(score.confidence, "language confidence")?;
            if score.confidence > previous
                || (score.confidence == previous
                    && previous_language.is_some_and(|value| value >= score.language.as_str()))
            {
                return crate::validation::invalid("language scores must be confidence ordered");
            }
            previous = score.confidence;
            previous_language = Some(&score.language);
        }
        Ok(())
    }
}
