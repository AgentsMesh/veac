use veac_artifact::ArtifactKind;
use veac_ir::{Rational, Rect};

use crate::test_support::*;
use crate::*;

#[test]
fn common_reviewable_values_validate_and_reject_bad_geometry() {
    ConfidenceLabel {
        label: "speech".into(),
        confidence: 0.8,
    }
    .validate()
    .unwrap();
    scalar().validate().unwrap();
    transform().validate().unwrap();
    crop().validate().unwrap();
    decision().validate().unwrap();

    let mut bad = crop();
    bad.rect = Rect {
        x: 0.8,
        y: 0.0,
        width: 0.5,
        height: 1.0,
    };
    assert!(bad.validate().is_err());
    let times = [time(1), time(1)];
    assert!(crate::validation::increasing(&times).is_err());
    let ranges = [range(10, 10), range(0, 5)];
    assert!(crate::validation::non_overlapping(&ranges).is_err());
}

#[test]
fn every_request_rejects_a_domain_specific_invalid_value() {
    for mut request in requests() {
        match &mut request {
            ProviderRequest::Asr(value) => value.audio.media_type = MediaType::Video,
            ProviderRequest::LanguageDetection(value) => {
                value.candidates = vec!["zh".into(), "en".into()]
            }
            ProviderRequest::Translation(value) => {
                value.target_language = value.source_language.clone()
            }
            ProviderRequest::TextToSpeech(value) => value.sample_rate = 0,
            ProviderRequest::Dubbing(value) => value.source_language.clear(),
            ProviderRequest::MotionTracking(value) => value.sample_interval.value = 0,
            ProviderRequest::Stabilization(value) => {
                value.motion_track.as_mut().unwrap().media_type = MediaType::Video
            }
            ProviderRequest::Segmentation(value) => value.subjects.clear(),
            ProviderRequest::Matte(value) => value.mattes.clear(),
            ProviderRequest::Denoise(value) => value.audio.media_type = MediaType::Video,
            ProviderRequest::VocalSeparation(value) => value.stems.clear(),
            ProviderRequest::SceneDetection(value) => value.minimum_scene_duration.value = 0,
            ProviderRequest::BeatDetection(value) => value.meter_hint = Some(0),
            ProviderRequest::SilenceDetection(value) => value.minimum_duration.value = 0,
            ProviderRequest::FillerDetection(value) => value.context_padding.value = -1,
            ProviderRequest::HighlightDetection(value) => value.inputs.clear(),
            ProviderRequest::AutoReframe(value) => value.maximum_speed_per_second = 0.0,
            ProviderRequest::Retouch(value) => value.targets.clear(),
            ProviderRequest::Removal(value) => value.targets.clear(),
            ProviderRequest::ColorMatch(value) => value.source.media_type = MediaType::Audio,
            ProviderRequest::MulticamSync(value) => value.angles.clear(),
        }
        assert!(request.validate().is_err(), "{:?}", request.capability());
    }
}

#[test]
fn every_output_rejects_a_domain_specific_invalid_value() {
    for payload in requests() {
        let request =
            ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
        let mut output = output_for(&request);
        match &mut output {
            ProviderOutput::Asr(value) => value.language.clear(),
            ProviderOutput::LanguageDetection(value) => value.languages[0].confidence = 2.0,
            ProviderOutput::Translation(value) => value.units[0].id.clear(),
            ProviderOutput::TextToSpeech(value) => {
                value.audio.role = ProviderArtifactSlot::new("wrong-role").unwrap()
            }
            ProviderOutput::Dubbing(value) => {
                value.turns[0].timing_scale = Rational {
                    numerator: 1,
                    denominator: 0,
                }
            }
            ProviderOutput::MotionTracking(value) => {
                value.track = artifact(
                    ArtifactKind::Speech,
                    "wrong-track",
                    &request.provider,
                    request_hash(&request).unwrap(),
                )
            }
            ProviderOutput::Stabilization(value) => value.crops[0].rect.width = 2.0,
            ProviderOutput::Segmentation(value) => {
                value.matte = artifact(
                    ArtifactKind::Speech,
                    "wrong-matte",
                    &request.provider,
                    request_hash(&request).unwrap(),
                )
            }
            ProviderOutput::Matte(value) => value.coverage[0].value = 2.0,
            ProviderOutput::Denoise(value) => value.residual_noise_db = f64::NAN,
            ProviderOutput::VocalSeparation(value) => value.stems.clear(),
            ProviderOutput::SceneDetection(value) => value.boundaries[0].confidence = 2.0,
            ProviderOutput::BeatDetection(value) => value.meter = 0,
            ProviderOutput::SilenceDetection(value) => value.intervals[0].mean_db = f64::NAN,
            ProviderOutput::FillerDetection(value) => value.occurrences[0].token.clear(),
            ProviderOutput::HighlightDetection(value) => {
                value.highlights[0].evidence = vec!["z".into(), "a".into()]
            }
            ProviderOutput::AutoReframe(value) => value.movements[0].end = value.movements[0].start,
            ProviderOutput::Retouch(value) => {
                value.masks[0] = artifact(
                    ArtifactKind::Speech,
                    "wrong-mask",
                    &request.provider,
                    request_hash(&request).unwrap(),
                )
            }
            ProviderOutput::Removal(value) => value.video.record.key.value.clear(),
            ProviderOutput::ColorMatch(value) => value.confidence = 2.0,
            ProviderOutput::MulticamSync(value) => value.offsets.clear(),
        }
        assert!(output.validate().is_err(), "{:?}", output.capability());
    }
}

#[test]
fn tracking_accepts_explicit_point_and_region_targets() {
    let mut value = match requests().remove(5) {
        ProviderRequest::MotionTracking(value) => value,
        _ => unreachable!(),
    };
    value.target = TrackingTarget::Point { x: 0.2, y: 0.3 };
    value.validate().unwrap();
    value.target = TrackingTarget::Region { rect: rect() };
    value.validate().unwrap();
}
