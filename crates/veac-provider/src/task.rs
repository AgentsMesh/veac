use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

mod output;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProviderRequest {
    Asr(AsrRequest),
    LanguageDetection(LanguageDetectionRequest),
    Translation(TranslationRequest),
    TextToSpeech(TtsRequest),
    Dubbing(DubbingRequest),
    MotionTracking(TrackingRequest),
    Stabilization(StabilizationRequest),
    Segmentation(SegmentationRequest),
    Matte(MatteRequest),
    Denoise(DenoiseRequest),
    VocalSeparation(VocalSeparationRequest),
    SceneDetection(SceneDetectionRequest),
    BeatDetection(BeatDetectionRequest),
    SilenceDetection(SilenceDetectionRequest),
    FillerDetection(FillerDetectionRequest),
    HighlightDetection(HighlightDetectionRequest),
    AutoReframe(AutoReframeRequest),
    Retouch(RetouchRequest),
    Removal(RemovalRequest),
    ColorMatch(ColorMatchRequest),
    MulticamSync(MulticamSyncRequest),
}

impl ProviderRequest {
    pub fn capability(&self) -> Capability {
        match self {
            Self::Asr(_) => Capability::Asr,
            Self::LanguageDetection(_) => Capability::LanguageDetection,
            Self::Translation(_) => Capability::Translation,
            Self::TextToSpeech(_) => Capability::TextToSpeech,
            Self::Dubbing(_) => Capability::Dubbing,
            Self::MotionTracking(_) => Capability::MotionTracking,
            Self::Stabilization(_) => Capability::Stabilization,
            Self::Segmentation(_) => Capability::Segmentation,
            Self::Matte(_) => Capability::Matte,
            Self::Denoise(_) => Capability::Denoise,
            Self::VocalSeparation(_) => Capability::VocalSeparation,
            Self::SceneDetection(_) => Capability::SceneDetection,
            Self::BeatDetection(_) => Capability::BeatDetection,
            Self::SilenceDetection(_) => Capability::SilenceDetection,
            Self::FillerDetection(_) => Capability::FillerDetection,
            Self::HighlightDetection(_) => Capability::HighlightDetection,
            Self::AutoReframe(_) => Capability::AutoReframe,
            Self::Retouch(_) => Capability::Retouch,
            Self::Removal(_) => Capability::Removal,
            Self::ColorMatch(_) => Capability::ColorMatch,
            Self::MulticamSync(_) => Capability::MulticamSync,
        }
    }
}

impl Validate for ProviderRequest {
    fn validate(&self) -> ProviderResult<()> {
        match self {
            Self::Asr(value) => value.validate(),
            Self::LanguageDetection(value) => value.validate(),
            Self::Translation(value) => value.validate(),
            Self::TextToSpeech(value) => value.validate(),
            Self::Dubbing(value) => value.validate(),
            Self::MotionTracking(value) => value.validate(),
            Self::Stabilization(value) => value.validate(),
            Self::Segmentation(value) => value.validate(),
            Self::Matte(value) => value.validate(),
            Self::Denoise(value) => value.validate(),
            Self::VocalSeparation(value) => value.validate(),
            Self::SceneDetection(value) => value.validate(),
            Self::BeatDetection(value) => value.validate(),
            Self::SilenceDetection(value) => value.validate(),
            Self::FillerDetection(value) => value.validate(),
            Self::HighlightDetection(value) => value.validate(),
            Self::AutoReframe(value) => value.validate(),
            Self::Retouch(value) => value.validate(),
            Self::Removal(value) => value.validate(),
            Self::ColorMatch(value) => value.validate(),
            Self::MulticamSync(value) => value.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProviderOutput {
    Asr(AsrResult),
    LanguageDetection(LanguageDetectionResult),
    Translation(TranslationResult),
    TextToSpeech(SpeechResult),
    Dubbing(DubbingResult),
    MotionTracking(TrackingResult),
    Stabilization(StabilizationResult),
    Segmentation(SegmentationResult),
    Matte(MatteResult),
    Denoise(DenoiseResult),
    VocalSeparation(VocalSeparationResult),
    SceneDetection(SceneDetectionResult),
    BeatDetection(BeatDetectionResult),
    SilenceDetection(SilenceDetectionResult),
    FillerDetection(FillerDetectionResult),
    HighlightDetection(HighlightDetectionResult),
    AutoReframe(AutoReframeResult),
    Retouch(RetouchResult),
    Removal(RemovalResult),
    ColorMatch(ColorMatchResult),
    MulticamSync(MulticamSyncResult),
}
