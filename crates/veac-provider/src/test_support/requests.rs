use veac_ir::Rational;

use super::*;

#[path = "requests/multicam.rs"]
mod multicam;
use multicam::multicam_request;

pub(crate) fn requests() -> Vec<ProviderRequest> {
    vec![
        ProviderRequest::Asr(AsrRequest {
            audio: input(MediaType::Audio),
            language_hint: Some("en-US".into()),
            speakers: SpeakerMode::KnownCount { count: 2 },
            word_timing: true,
        }),
        ProviderRequest::LanguageDetection(LanguageDetectionRequest {
            input: input(MediaType::Audio),
            candidates: vec!["en".into(), "zh-Hans".into()],
        }),
        ProviderRequest::Translation(TranslationRequest {
            source_language: "en".into(),
            target_language: "zh-Hans".into(),
            units: vec![TranslationUnit {
                id: "unit-1".into(),
                range: Some(range(0, 10)),
                text: "hello".into(),
            }],
            preserve_timing: true,
        }),
        ProviderRequest::TextToSpeech(TtsRequest {
            text: "hello".into(),
            voice: voice(),
            sample_rate: 48_000,
            target_range: Some(range(0, 10)),
        }),
        ProviderRequest::Dubbing(DubbingRequest {
            audio: input(MediaType::Audio),
            source_language: "en".into(),
            target_language: "zh".into(),
            turns: vec![DubbingTurn {
                id: "turn-1".into(),
                source_range: range(0, 10),
                source_text: "hello".into(),
                translated_text: "ni hao".into(),
                voice: voice(),
            }],
            preserve_timing: true,
        }),
        ProviderRequest::MotionTracking(TrackingRequest {
            video: input(MediaType::Video),
            mode: TrackingMode::Object,
            target: TrackingTarget::Semantic {
                label: "speaker".into(),
                initial_rect: rect(),
            },
            sample_interval: time(1),
            smoothing: 0.5,
        }),
        ProviderRequest::Stabilization(StabilizationRequest {
            video: input(MediaType::Video),
            motion_track: Some(input(MediaType::Analysis)),
            strength: 0.8,
            crop_margin: 0.1,
        }),
        ProviderRequest::Segmentation(SegmentationRequest {
            video: input(MediaType::Video),
            subjects: vec![SubjectPrompt {
                id: "subject-1".into(),
                label: "person".into(),
                initial_rect: Some(rect()),
            }],
            temporal_consistency: 0.8,
            edge_refinement: 0.5,
            correction_mattes: vec![input(MediaType::Matte)],
        }),
        ProviderRequest::Matte(MatteRequest {
            mattes: vec![input(MediaType::Matte)],
            operation: MatteOperation::Union,
            feather_pixels: 1.0,
            expansion_pixels: 0.0,
        }),
        ProviderRequest::Denoise(DenoiseRequest {
            audio: input(MediaType::Audio),
            strength: 0.7,
            preserve_voice: true,
            noise_only_ranges: vec![range(0, 10)],
        }),
        ProviderRequest::VocalSeparation(VocalSeparationRequest {
            audio: input(MediaType::Audio),
            stems: vec![StemKind::Vocals, StemKind::Music],
            preserve_phase: true,
        }),
        ProviderRequest::SceneDetection(SceneDetectionRequest {
            video: input(MediaType::Video),
            sensitivity: 0.5,
            minimum_scene_duration: time(1),
        }),
        ProviderRequest::BeatDetection(BeatDetectionRequest {
            audio: input(MediaType::Audio),
            tempo_range: Some((80.0, 160.0)),
            meter_hint: Some(4),
        }),
        ProviderRequest::SilenceDetection(SilenceDetectionRequest {
            audio: input(MediaType::Audio),
            threshold_db: -40.0,
            minimum_duration: time(10),
        }),
        filler_request(),
        highlight_request(),
        reframe_request(),
        retouch_request(),
        removal_request(),
        color_request(),
        multicam_request(),
    ]
}

pub(crate) fn voice() -> VoiceSpec {
    VoiceSpec {
        voice: "voice-1".into(),
        language: "en".into(),
        speaking_rate: Rational::new(1, 1).unwrap(),
        pitch_semitones: 0.0,
    }
}

fn filler_request() -> ProviderRequest {
    ProviderRequest::FillerDetection(FillerDetectionRequest {
        transcript: input(MediaType::Text),
        language: "en".into(),
        filler_lexicon: vec!["um".into()],
        context_padding: time(0),
    })
}

fn highlight_request() -> ProviderRequest {
    ProviderRequest::HighlightDetection(HighlightDetectionRequest {
        inputs: vec![input(MediaType::Video)],
        objective: "most informative".into(),
        target_duration: Some(time(50)),
        maximum_highlights: 3,
    })
}

fn reframe_request() -> ProviderRequest {
    ProviderRequest::AutoReframe(AutoReframeRequest {
        video: input(MediaType::Video),
        target_aspect_ratio: Rational::new(9, 16).unwrap(),
        subject_tracks: vec![input(MediaType::Analysis)],
        smoothing: 0.6,
        dead_zone: 0.2,
        maximum_speed_per_second: 0.5,
        manual_overrides: vec![crop()],
    })
}

fn retouch_request() -> ProviderRequest {
    ProviderRequest::Retouch(RetouchRequest {
        video: input(MediaType::Video),
        targets: vec![RetouchTarget {
            id: "face-1".into(),
            kind: RetouchKind::Face,
            initial_rect: rect(),
        }],
        controls: vec![ParameterCurve {
            parameter: "skin_smoothing".into(),
            samples: vec![scalar()],
        }],
        correction_mattes: vec![input(MediaType::Matte)],
    })
}

fn removal_request() -> ProviderRequest {
    ProviderRequest::Removal(RemovalRequest {
        video: input(MediaType::Video),
        targets: vec![RemovalTarget {
            id: "logo-1".into(),
            label: "logo".into(),
            range: range(0, 10),
            initial_rect: rect(),
        }],
        fill: RemovalFill::Temporal,
        correction_mattes: vec![input(MediaType::Matte)],
    })
}

fn color_request() -> ProviderRequest {
    ProviderRequest::ColorMatch(ColorMatchRequest {
        source: input(MediaType::Video),
        reference: input(MediaType::Image),
        source_samples: vec![range(0, 10)],
        reference_samples: vec![range(0, 10)],
        intent: ColorMatchIntent::ReferenceLook,
        protected_colors: vec![],
    })
}
