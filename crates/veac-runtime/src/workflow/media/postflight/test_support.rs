use veac_artifact::*;
use veac_ir::*;

#[path = "test_support/streams.rs"]
mod streams;
pub(super) use streams::time;
use streams::{audio, identity, rate, selection, video};

pub(super) fn valid_cases() -> Vec<(MediaArtifactSpec, MediaProbeSnapshot)> {
    vec![
        pair(
            MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
                source_stream: selection(0),
                source_clock: identity(100),
                width: 320,
                height: 180,
                frame_rate: rate(30),
                crf: 24,
            }),
            vec![video(
                "h264",
                320,
                180,
                Some(rate(30)),
                Some(time(100)),
                true,
            )],
        ),
        pair(
            audio_spec(),
            vec![audio("pcm_s16le", 48_000, 2, Some(time(100)), false)],
        ),
        pair(
            MediaArtifactSpec::Waveform(WaveformSpec {
                source_stream: selection(1),
                source_clock: identity(100),
                sample_rate: 24_000,
                width: 400,
                height: 80,
                color: "white".into(),
            }),
            vec![video("png", 400, 80, Some(rate(25)), None, false)],
        ),
        pair(
            MediaArtifactSpec::Thumbnail(ThumbnailSpec {
                source_stream: selection(0),
                at: time(50),
                width: 160,
                height: 90,
            }),
            vec![video("png", 160, 90, Some(rate(25)), None, false)],
        ),
        pair(
            MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
                source_stream: selection(0),
                source_clock: identity(100),
                width: 320,
                height: 180,
                frame_rate: rate(60),
                method: OpticalFlowMethod::MotionCompensated,
            }),
            vec![video(
                "h264",
                320,
                180,
                Some(rate(60)),
                Some(time(100)),
                true,
            )],
        ),
        pair(
            MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
                video_stream: selection(0),
                start: time(20),
                duration: time(50),
                width: 320,
                height: 180,
                frame_rate: rate(30),
                audio: Some(SourceSegmentAudioSpec {
                    source_stream: selection(1),
                    sample_rate: 48_000,
                    channels: 2,
                }),
                crf: 24,
            }),
            vec![
                video("h264", 320, 180, Some(rate(30)), Some(time(50)), true),
                audio("aac", 48_000, 2, Some(time(50)), true),
            ],
        ),
    ]
}

pub(super) fn audio_spec() -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: selection(1),
        source_clock: identity(100),
        sample_rate: 48_000,
        channels: 2,
    })
}

pub(super) fn analysis_case() -> (MediaArtifactSpec, MediaProbeSnapshot) {
    pair(
        MediaArtifactSpec::Analysis(AnalysisSpec {
            analysis_type: "scene".into(),
            configuration: serde_json::json!({}),
        }),
        vec![video("png", 1, 1, Some(rate(1)), None, false)],
    )
}

fn pair(
    spec: MediaArtifactSpec,
    streams: Vec<ProbedStream>,
) -> (MediaArtifactSpec, MediaProbeSnapshot) {
    let video = streams
        .iter()
        .find(|value| value.media_type == ProbedStreamType::Video);
    let audio = streams
        .iter()
        .find(|value| value.media_type == ProbedStreamType::Audio);
    let probe = MediaProbeSnapshot {
        schema_version: MEDIA_PROBE_SCHEMA_VERSION,
        engine: "test".into(),
        selection_policy: "test".into(),
        container_format: "test".into(),
        container_brand: None,
        observed_identity: MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: "ab".repeat(32),
        },
        container_duration: None,
        selected_video_stream: video.map(|value| selection(value.global_index)),
        selected_audio_stream: audio.map(|value| selection(value.global_index)),
        streams,
    };
    (spec, probe)
}
