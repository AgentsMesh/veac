use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};
use veac_runtime::asset::{selected_audio, selected_video};
use veac_runtime::workflow::MediaWorkflow;

use super::support::*;

#[test]
fn real_ffmpeg_derives_and_revalidates_all_six_media_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let input = media_fixture(temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let workflow = MediaWorkflow::new("ffmpeg");
    for (index, spec) in specs().into_iter().enumerate() {
        let request = request(&input, spec);
        let descriptor = request.descriptor().unwrap();
        let created = workflow
            .derive(&store, &input, &request)
            .unwrap_or_else(|error| panic!("artifact {index} failed: {error:?}"));
        assert!(!created.cache_hit);
        let facts = probe_artifact(&store, &descriptor, &created.record);
        assert_contract(index, &facts);

        let cached = workflow.derive(&store, &input, &request).unwrap();
        assert!(cached.cache_hit);
        assert_eq!(cached.record, created.record);
    }
}

fn specs() -> [MediaArtifactSpec; 6] {
    let video = stream(1, 1);
    let audio = stream(3, 1);
    [
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: video,
            source_clock: bounded(25, 50),
            width: 120,
            height: 68,
            frame_rate: rate(12),
            crf: 26,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: audio,
            source_clock: identity(),
            sample_rate: 24_000,
            channels: 1,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: audio,
            source_clock: bounded(10, 50),
            sample_rate: 24_000,
            width: 320,
            height: 80,
            color: "blue".into(),
        }),
        MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: video,
            at: time(75),
            width: 120,
            height: 68,
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: video,
            source_clock: identity(),
            width: 120,
            height: 68,
            frame_rate: rate(24),
            method: OpticalFlowMethod::BlockMatching,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: video,
            start: time(10),
            duration: time(40),
            width: 128,
            height: 72,
            frame_rate: rate(12),
            audio: Some(SourceSegmentAudioSpec {
                source_stream: audio,
                sample_rate: 24_000,
                channels: 1,
            }),
            crf: 24,
        }),
    ]
}

fn assert_contract(index: usize, probe: &veac_ir::MediaProbeSnapshot) {
    match index {
        0 => assert_video(probe, 120, 68, rate(12), Some(0.5), false),
        1 => assert_audio(probe, "pcm_s16le", 24_000, 1, 1.0),
        2 => assert_video(probe, 320, 80, rate(25), None, false),
        3 => assert_video(probe, 120, 68, rate(25), None, false),
        4 => assert_video(probe, 120, 68, rate(24), Some(1.0), false),
        5 => {
            assert_video(probe, 128, 72, rate(12), Some(0.4), true);
            assert_audio(probe, "aac", 24_000, 1, 0.4);
        }
        _ => unreachable!(),
    }
}

fn assert_video(
    probe: &veac_ir::MediaProbeSnapshot,
    width: u32,
    height: u32,
    frame_rate: Rational,
    duration: Option<f64>,
    has_audio: bool,
) {
    assert_eq!(probe.streams.len(), 1 + usize::from(has_audio));
    let stream = selected_video(probe).unwrap();
    let video = stream.video.as_ref().unwrap();
    assert_eq!(
        stream.codec,
        if duration.is_none() { "png" } else { "h264" }
    );
    assert_eq!((video.width, video.height), (width, height));
    if duration.is_some() {
        assert_eq!(video.frame_rate, Some(frame_rate));
        assert_eq!(stream.start_time.unwrap().value, 0);
    } else {
        assert!(stream.start_time.is_none_or(|value| value.value == 0));
    }
    if let Some(expected) = duration {
        assert_duration(
            stream.duration.or(probe.container_duration),
            expected,
            1.0 / 12.0,
        );
    }
}

fn assert_audio(
    probe: &veac_ir::MediaProbeSnapshot,
    codec: &str,
    sample_rate: u32,
    channels: u8,
    duration: f64,
) {
    let stream = selected_audio(probe).unwrap();
    let audio = stream.audio.as_ref().unwrap();
    assert_eq!(stream.codec, codec);
    assert_eq!((audio.sample_rate, audio.channels), (sample_rate, channels));
    if codec == "pcm_s16le" {
        assert!(stream.start_time.is_none_or(|value| value.value == 0));
    } else {
        assert_eq!(stream.start_time.unwrap().value, 0);
    }
    assert_duration(
        stream.duration.or(probe.container_duration),
        duration,
        0.002,
    );
}

fn assert_duration(actual: Option<RationalTime>, expected: f64, slack: f64) {
    let actual = actual.unwrap();
    let seconds = actual.value as f64 / f64::from(actual.timescale);
    assert!(
        (seconds - expected).abs() <= slack,
        "{seconds} vs {expected}"
    );
}

fn identity() -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: time(100),
    }
}

fn bounded(start: i64, duration: i64) -> SourceClockSpec {
    SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(start), time(duration)).unwrap(),
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn rate(value: i64) -> Rational {
    Rational::new(value, 1).unwrap()
}

fn stream(global_index: u32, type_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index,
    }
}
