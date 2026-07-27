use std::path::{Path, PathBuf};

use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};

pub(super) fn specs() -> Vec<MediaArtifactSpec> {
    let video = selection(0);
    let audio = selection(1);
    vec![
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: video,
            source_clock: bounded(10, 50),
            width: 160,
            height: 90,
            frame_rate: rate(15),
            crf: 24,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: audio,
            source_clock: identity(),
            sample_rate: 24_000,
            channels: 1,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: audio,
            source_clock: bounded(20, 40),
            sample_rate: 24_000,
            width: 320,
            height: 80,
            color: "white".into(),
        }),
        MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: video,
            at: time(50),
            width: 160,
            height: 90,
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: video,
            source_clock: identity(),
            width: 160,
            height: 90,
            frame_rate: rate(30),
            method: OpticalFlowMethod::BlockMatching,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: video,
            start: time(20),
            duration: time(50),
            width: 160,
            height: 90,
            frame_rate: rate(15),
            audio: Some(SourceSegmentAudioSpec {
                source_stream: audio,
                sample_rate: 24_000,
                channels: 1,
            }),
            crf: 24,
        }),
    ]
}

pub(super) fn request(source: &[u8], spec: MediaArtifactSpec) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity: ContentDigest::sha256(source),
        producer: ProducerFingerprint {
            name: "test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"config"),
        },
        spec,
    }
}

pub(super) fn ffprobe(root: &Path) -> PathBuf {
    super::super::test_support::executable(
        root.join("preflight-ffprobe.sh"),
        r#"#!/bin/sh
if [ "$1" = "-version" ]; then printf 'ffprobe version preflight\n'; exit 0; fi
printf '%s' '{"format":{"format_name":"mov,mp4","duration":"2"},"streams":[{"index":0,"codec_type":"video","codec_name":"h264","time_base":"1/30","start_time":"0","duration":"2","width":320,"height":180,"avg_frame_rate":"30/1","r_frame_rate":"30/1","pix_fmt":"yuv420p","sample_aspect_ratio":"1:1"},{"index":1,"codec_type":"audio","codec_name":"aac","time_base":"1/48000","start_time":"0","duration":"2","sample_rate":"48000","channels":2,"channel_layout":"stereo"}]}'
"#,
    )
}

pub(super) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
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

fn selection(global_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index: 0,
    }
}

fn rate(value: i64) -> Rational {
    Rational::new(value, 1).unwrap()
}
