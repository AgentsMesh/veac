use veac_artifact::{MediaArtifactSpec, ProxyAudioSpec, ProxyVideoSpec};
use veac_ir::{StreamChoice, StreamIntent, StreamSelection};

pub(super) fn for_spec(spec: &MediaArtifactSpec) -> StreamIntent {
    let disabled = StreamChoice::Disabled;
    let (video, audio) = match spec {
        MediaArtifactSpec::ProxyVideo(value) => video(value, disabled),
        MediaArtifactSpec::OpticalFlow(value) => (choice(value.source_stream), disabled),
        MediaArtifactSpec::ProxyAudio(value) => audio(value, disabled),
        MediaArtifactSpec::Waveform(value) => (disabled, choice(value.source_stream)),
        MediaArtifactSpec::Thumbnail(value) => (choice(value.source_stream), disabled),
        MediaArtifactSpec::SourceSegment(value) => (
            choice(value.video_stream),
            value
                .audio
                .map_or(disabled, |audio| choice(audio.source_stream)),
        ),
        MediaArtifactSpec::Analysis(_) => (disabled, disabled),
    };
    StreamIntent { video, audio }
}

fn video(value: &ProxyVideoSpec, disabled: StreamChoice) -> (StreamChoice, StreamChoice) {
    (choice(value.source_stream), disabled)
}

fn audio(value: &ProxyAudioSpec, disabled: StreamChoice) -> (StreamChoice, StreamChoice) {
    (disabled, choice(value.source_stream))
}

fn choice(value: StreamSelection) -> StreamChoice {
    StreamChoice::GlobalIndex {
        global_index: value.global_index,
    }
}
