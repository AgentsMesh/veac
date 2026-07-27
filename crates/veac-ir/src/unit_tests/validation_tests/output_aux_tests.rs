#[path = "output_aux_tests/audio_stem_tests.rs"]
mod audio_stem_tests;
#[path = "output_aux_tests/caption_tests.rs"]
mod caption_tests;
#[path = "output_aux_tests/image_tests.rs"]
mod image_tests;
#[path = "output_aux_tests/scope_tests.rs"]
mod scope_tests;

use super::*;

fn deliverable(file_name: &str, kind: DeliverableKind) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_aux").unwrap(),
        file_name: file_name.to_owned(),
        kind,
    }
}

fn project_with(value: Deliverable) -> ProjectEnvelope {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables = vec![value];
    project
}

fn assert_valid(value: Deliverable) {
    validate(&project_with(value)).unwrap();
}

fn assert_invalid(value: Deliverable, code: &str) {
    assert_code(&validation_codes(&project_with(value)), code);
}

fn audio(codec: AudioCodec) -> AudioOutput {
    AudioOutput {
        codec,
        sample_rate: 48_000,
        channels: 2,
    }
}

fn image(file_name: &str, format: ImageFormat) -> Deliverable {
    deliverable(
        file_name,
        DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number: 1,
        }),
    )
}

fn caption(file_name: &str, format: CaptionSidecarFormat, tracks: &[&str]) -> Deliverable {
    deliverable(
        file_name,
        DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format,
            track_ids: tracks.iter().map(|id| TrackId::new(*id).unwrap()).collect(),
        }),
    )
}

fn stem(
    file_name: &str,
    format: AudioStemFormat,
    codec: AudioCodec,
    source: AudioStemSource,
) -> Deliverable {
    deliverable(
        file_name,
        DeliverableKind::AudioStem(AudioStemOutput {
            format,
            audio: audio(codec),
            source,
        }),
    )
}

fn scope(file_name: &str, format: ImageFormat, scope: VideoScope) -> Deliverable {
    deliverable(
        file_name,
        DeliverableKind::Scope(ScopeOutput {
            scope,
            at: RationalTime::new(0, 600).unwrap(),
            width: 320,
            height: 180,
            format,
        }),
    )
}
