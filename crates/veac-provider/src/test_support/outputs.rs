use veac_artifact::ArtifactKind;
use veac_ir::Rational;

use super::*;

pub(crate) fn output_for(request: &ProviderRequestEnvelope) -> ProviderOutput {
    let hash = request_hash(request).unwrap();
    let artifact = |kind, role| artifact(kind, role, &request.provider, hash.clone());
    match request.capability {
        Capability::Asr => ProviderOutput::Asr(AsrResult {
            language: "en".into(),
            segments: vec![TranscriptSegment {
                id: "segment-1".into(),
                range: range(0, 10),
                speaker: Some("speaker-1".into()),
                text: "hello".into(),
                confidence: 0.9,
                words: vec![word()],
            }],
        }),
        Capability::LanguageDetection => {
            ProviderOutput::LanguageDetection(LanguageDetectionResult {
                languages: vec![LanguageScore {
                    language: "en".into(),
                    confidence: 0.9,
                }],
            })
        }
        Capability::Translation => ProviderOutput::Translation(TranslationResult {
            source_language: "en".into(),
            target_language: "zh".into(),
            units: vec![TranslatedUnit {
                id: "unit-1".into(),
                range: Some(range(0, 10)),
                text: "ni hao".into(),
                words: vec![word()],
            }],
        }),
        Capability::TextToSpeech => ProviderOutput::TextToSpeech(SpeechResult {
            audio: artifact(ArtifactKind::Speech, "speech"),
            range: range(0, 10),
            words: vec![word()],
        }),
        Capability::Dubbing => ProviderOutput::Dubbing(DubbingResult {
            audio: artifact(ArtifactKind::Speech, "dub"),
            turns: vec![DubbedTurn {
                id: "turn-1".into(),
                source_range: range(0, 10),
                rendered_range: range(0, 10),
                timing_scale: Rational::new(1, 1).unwrap(),
            }],
        }),
        Capability::MotionTracking => ProviderOutput::MotionTracking(TrackingResult {
            track: artifact(ArtifactKind::MotionTrack, "track"),
            transforms: vec![transform()],
            regions: vec![TrackedRegion {
                time: time(0),
                rect: rect(),
                confidence: 0.9,
            }],
            lost_ranges: vec![range(20, 10)],
        }),
        Capability::Stabilization => ProviderOutput::Stabilization(StabilizationResult {
            transforms: vec![transform()],
            crops: vec![crop()],
            decisions: vec![decision()],
        }),
        Capability::Segmentation => ProviderOutput::Segmentation(SegmentationResult {
            matte: artifact(ArtifactKind::Matte, "subject-matte"),
            subjects: vec![SubjectVisibility {
                subject_id: "subject-1".into(),
                visible_ranges: vec![range(0, 10)],
                confidence: 0.9,
            }],
            quality: vec![scalar()],
            decisions: vec![decision()],
        }),
        Capability::Matte => ProviderOutput::Matte(MatteResult {
            matte: artifact(ArtifactKind::Matte, "combined-matte"),
            coverage: vec![scalar()],
        }),
        Capability::Denoise => ProviderOutput::Denoise(DenoiseResult {
            audio: artifact(ArtifactKind::AudioStem, "denoised-audio"),
            noise_reduction_db: vec![scalar()],
            residual_noise_db: -52.0,
        }),
        Capability::VocalSeparation => ProviderOutput::VocalSeparation(VocalSeparationResult {
            stems: vec![SeparatedStem {
                kind: StemKind::Vocals,
                label: "vocals".into(),
                audio: artifact(ArtifactKind::AudioStem, "vocals"),
                leakage: 0.05,
            }],
        }),
        Capability::SceneDetection => ProviderOutput::SceneDetection(SceneDetectionResult {
            boundaries: vec![SceneBoundary {
                time: time(10),
                confidence: 0.9,
                hard_cut: true,
            }],
            scenes: vec![range(0, 10)],
        }),
        Capability::BeatDetection => ProviderOutput::BeatDetection(BeatDetectionResult {
            tempo: Rational::new(120, 1).unwrap(),
            meter: 4,
            beats: vec![Beat {
                time: time(0),
                confidence: 0.9,
                bar: 1,
                beat_in_bar: 1,
            }],
        }),
        Capability::SilenceDetection => ProviderOutput::SilenceDetection(SilenceDetectionResult {
            intervals: vec![DetectedSilence {
                range: range(0, 10),
                mean_db: -60.0,
                confidence: 0.9,
            }],
        }),
        Capability::FillerDetection => filler_output(),
        Capability::HighlightDetection => highlight_output(),
        Capability::AutoReframe => reframe_output(),
        Capability::Retouch => ProviderOutput::Retouch(RetouchResult {
            masks: vec![artifact(ArtifactKind::Matte, "retouch-mask")],
            controls: vec![ParameterCurve {
                parameter: "skin_smoothing".into(),
                samples: vec![scalar()],
            }],
            decisions: vec![decision()],
        }),
        Capability::Removal => ProviderOutput::Removal(RemovalResult {
            video: artifact(ArtifactKind::VideoMaster, "clean-video"),
            masks: vec![artifact(ArtifactKind::Matte, "removal-mask")],
            decisions: vec![decision()],
        }),
        Capability::ColorMatch => ProviderOutput::ColorMatch(ColorMatchResult {
            adjustment: ColorAdjustment {
                matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                offset: [0.0; 3],
                exposure_stops: 0.1,
                temperature_kelvin: 6500.0,
                tint: 0.0,
                contrast: 1.0,
                saturation: 1.0,
            },
            confidence: 0.8,
            decisions: vec![decision()],
        }),
        Capability::MulticamSync => ProviderOutput::MulticamSync(MulticamSyncResult {
            basis: veac_ir::MulticamSyncBasis::Audio,
            reference_angle_id: veac_ir::MulticamAngleId::new("ang_a").unwrap(),
            offsets: vec![
                MulticamSyncOffset {
                    angle_id: veac_ir::MulticamAngleId::new("ang_a").unwrap(),
                    material_id: veac_ir::MaterialId::new("med_source_video").unwrap(),
                    source_offset: time(0),
                    confidence: 1.0,
                },
                MulticamSyncOffset {
                    angle_id: veac_ir::MulticamAngleId::new("ang_b").unwrap(),
                    material_id: veac_ir::MaterialId::new("med_source_video_b").unwrap(),
                    source_offset: time(10),
                    confidence: 0.92,
                },
            ],
            decisions: vec![decision()],
        }),
    }
}
