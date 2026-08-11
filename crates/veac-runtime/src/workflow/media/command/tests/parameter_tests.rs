use super::*;
use veac_ir::TimeRange;

#[test]
fn every_explicit_spec_parameter_reaches_ffmpeg_arguments() {
    let video = args(&MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
        source_stream: stream(2),
        source_clock: bounded(25, 50),
        width: 640,
        height: 360,
        frame_rate: Rational::new(30, 1).unwrap(),
        crf: 23,
    }))
    .unwrap();
    assert_pairs(
        &video,
        &[
            ("-ss", "0.25"),
            ("-t", "0.5"),
            ("-map", "0:2"),
            ("-r", "30/1"),
            ("-crf", "23"),
        ],
    );
    assert_contains(&video, "scale=640:360");

    let audio = args(&MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: stream(3),
        source_clock: bounded(25, 50),
        sample_rate: 48_000,
        channels: 2,
    }))
    .unwrap();
    assert_pairs(&audio, &[("-map", "0:3"), ("-ar", "48000"), ("-ac", "2")]);

    let waveform = args(&MediaArtifactSpec::Waveform(WaveformSpec {
        source_stream: stream(4),
        source_clock: bounded(25, 50),
        sample_rate: 24_000,
        width: 800,
        height: 120,
        color: "#12ab34".into(),
    }))
    .unwrap();
    assert_contains(&waveform, "[0:4]aresample=24000");
    assert_contains(&waveform, "showwavespic=s=800x120:colors=#12ab34");

    let thumbnail = args(&MediaArtifactSpec::Thumbnail(ThumbnailSpec {
        source_stream: stream(5),
        at: time(125),
        width: 320,
        height: 180,
    }))
    .unwrap();
    assert_pairs(&thumbnail, &[("-ss", "1.25"), ("-map", "0:5")]);
    assert_contains(&thumbnail, "scale=320:180");

    let optical = args(&MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
        source_stream: stream(6),
        source_clock: bounded(25, 50),
        width: 640,
        height: 360,
        frame_rate: Rational::new(60, 1).unwrap(),
        method: OpticalFlowMethod::MotionCompensated,
    }))
    .unwrap();
    assert_pair(&optical, "-map", "0:6");
    assert_contains(&optical, "trim=start=0.25:duration=0.5");
    assert_contains(&optical, "scale=640:360");
    assert_contains(&optical, "minterpolate=fps=60/1:mi_mode=mci:mc_mode=aobmc");

    let segment = args(&MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
        video_stream: stream(7),
        start: time(10),
        duration: time(50),
        width: 1920,
        height: 1080,
        frame_rate: Rational::new(30, 1).unwrap(),
        audio: Some(SourceSegmentAudioSpec {
            source_stream: stream(8),
            sample_rate: 48_000,
            channels: 2,
        }),
        crf: 18,
    }))
    .unwrap();
    assert_pairs(
        &segment,
        &[
            ("-ss", "0.1"),
            ("-t", "0.5"),
            ("-map", "0:7"),
            ("-map", "0:8"),
            ("-r", "30/1"),
            ("-ar", "48000"),
            ("-ac", "2"),
            ("-crf", "18"),
        ],
    );
    assert_contains(&segment, "scale=1920:1080");
}

fn bounded(start: i64, duration: i64) -> SourceClockSpec {
    SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(start), time(duration)).unwrap(),
    }
}

fn assert_pairs(values: &[String], expected: &[(&str, &str)]) {
    for (name, value) in expected {
        assert_pair(values, name, value);
    }
}

fn assert_contains(values: &[String], expected: &str) {
    assert!(values.iter().any(|value| value.contains(expected)));
}
