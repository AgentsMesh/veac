use crate::*;

impl ProviderOutput {
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

    pub fn artifacts(&self) -> Vec<&ProviderArtifact> {
        match self {
            Self::TextToSpeech(value) => vec![&value.audio],
            Self::Dubbing(value) => vec![&value.audio],
            Self::MotionTracking(value) => vec![&value.track],
            Self::Segmentation(value) => vec![&value.matte],
            Self::Matte(value) => vec![&value.matte],
            Self::Denoise(value) => vec![&value.audio],
            Self::VocalSeparation(value) => value.stems.iter().map(|stem| &stem.audio).collect(),
            Self::Retouch(value) => value.masks.iter().collect(),
            Self::Removal(value) => std::iter::once(&value.video)
                .chain(value.masks.iter())
                .collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn artifacts_mut(&mut self) -> Vec<&mut ProviderArtifact> {
        match self {
            Self::TextToSpeech(value) => vec![&mut value.audio],
            Self::Dubbing(value) => vec![&mut value.audio],
            Self::MotionTracking(value) => vec![&mut value.track],
            Self::Segmentation(value) => vec![&mut value.matte],
            Self::Matte(value) => vec![&mut value.matte],
            Self::Denoise(value) => vec![&mut value.audio],
            Self::VocalSeparation(value) => {
                value.stems.iter_mut().map(|stem| &mut stem.audio).collect()
            }
            Self::Retouch(value) => value.masks.iter_mut().collect(),
            Self::Removal(value) => std::iter::once(&mut value.video)
                .chain(value.masks.iter_mut())
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl Validate for ProviderOutput {
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
