use crate::test_support::*;
use crate::*;

#[test]
fn requests_reject_inexact_media_roles() {
    for mut request in requests() {
        let selected = match &mut request {
            ProviderRequest::SceneDetection(value) => {
                value.video.media_type = MediaType::Audio;
                true
            }
            ProviderRequest::BeatDetection(value) => {
                value.audio.media_type = MediaType::Video;
                true
            }
            ProviderRequest::FillerDetection(value) => {
                value.transcript.media_type = MediaType::Audio;
                true
            }
            ProviderRequest::Dubbing(value) => {
                value.audio.media_type = MediaType::Video;
                true
            }
            ProviderRequest::VocalSeparation(value) => {
                value.audio.media_type = MediaType::Video;
                true
            }
            ProviderRequest::AutoReframe(value) => {
                value.subject_tracks.push(input(MediaType::Video));
                true
            }
            ProviderRequest::Segmentation(value) => {
                value.correction_mattes[0].media_type = MediaType::Video;
                true
            }
            ProviderRequest::Matte(value) => {
                value.mattes[0].media_type = MediaType::Video;
                true
            }
            ProviderRequest::Retouch(value) => {
                value.correction_mattes[0].media_type = MediaType::Video;
                true
            }
            ProviderRequest::ColorMatch(value) => {
                value.reference.media_type = MediaType::Audio;
                true
            }
            _ => false,
        };
        if selected {
            assert_invalid(&request);
        }
    }
}

#[test]
fn requests_reject_duplicate_bindings_and_zero_optional_limits() {
    for mut request in requests() {
        let selected = match &mut request {
            ProviderRequest::Translation(value) => {
                value.units.push(value.units[0].clone());
                true
            }
            ProviderRequest::Dubbing(value) => {
                value.turns.push(value.turns[0].clone());
                true
            }
            ProviderRequest::FillerDetection(value) => {
                value.filler_lexicon = vec!["um".into(), "um".into()];
                true
            }
            ProviderRequest::HighlightDetection(value) => {
                value.target_duration = Some(time(0));
                true
            }
            ProviderRequest::Segmentation(value) => {
                value.subjects.push(value.subjects[0].clone());
                true
            }
            ProviderRequest::Retouch(value) => {
                value.targets.push(value.targets[0].clone());
                true
            }
            _ => false,
        };
        if selected {
            assert_invalid(&request);
        }
    }

    let mut request = request(Capability::Retouch);
    let ProviderRequest::Retouch(value) = &mut request else {
        unreachable!()
    };
    value.controls.push(value.controls[0].clone());
    assert_invalid(&request);
}

#[test]
fn outputs_reject_nested_ordering_and_timing_violations() {
    for payload in requests() {
        let request =
            ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
        let mut output = output_for(&request);
        let selected = match &mut output {
            ProviderOutput::Asr(value) => {
                value.segments[0].speaker = Some(String::new());
                true
            }
            ProviderOutput::Translation(value) => {
                value.target_language.clone_from(&value.source_language);
                true
            }
            ProviderOutput::BeatDetection(value) => {
                value.beats[0].beat_in_bar = 0;
                true
            }
            ProviderOutput::FillerDetection(value) => {
                value.occurrences.push(value.occurrences[0].clone());
                true
            }
            ProviderOutput::HighlightDetection(value) => {
                value.highlights.push(value.highlights[0].clone());
                true
            }
            ProviderOutput::VocalSeparation(value) => {
                value.stems.push(value.stems[0].clone());
                true
            }
            ProviderOutput::Segmentation(value) => {
                value.subjects.push(value.subjects[0].clone());
                true
            }
            ProviderOutput::Retouch(value) => {
                value.controls.push(value.controls[0].clone());
                true
            }
            ProviderOutput::AutoReframe(value) => {
                value.crops.push(value.crops[0].clone());
                true
            }
            _ => false,
        };
        if selected {
            assert_invalid(&output);
        }
    }
}

fn request(capability: Capability) -> ProviderRequest {
    requests()
        .into_iter()
        .find(|value| value.capability() == capability)
        .unwrap()
}

fn assert_invalid(value: &impl Validate) {
    assert_eq!(
        value.validate().unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
}
