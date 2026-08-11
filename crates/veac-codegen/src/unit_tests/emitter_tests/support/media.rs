use std::path::Path;

use veac_plan::canonical::*;

pub fn identity(fill: char) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: fill.to_string().repeat(64),
    }
}

pub fn file_identity(path: &Path) -> MediaIdentity {
    let bytes = std::fs::read(path).expect("read test resource");
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: veac_artifact::ContentDigest::sha256(bytes).value,
    }
}

pub fn probe(identity: MediaIdentity) -> MediaProbeSnapshot {
    MediaProbeSnapshot {
        schema_version: MEDIA_PROBE_SCHEMA_VERSION,
        engine: "ffprobe version 8.0".to_owned(),
        selection_policy: "veac-stream-selection-v1".to_owned(),
        container_format: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
        container_brand: Some("isom".to_owned()),
        observed_identity: identity,
        container_duration: Some(time(6_000)),
        streams: vec![video_stream(), audio_stream()],
        selected_video_stream: Some(StreamSelection {
            global_index: 2,
            type_index: 0,
        }),
        selected_audio_stream: Some(StreamSelection {
            global_index: 5,
            type_index: 0,
        }),
    }
}

pub fn video_stream() -> ProbedStream {
    ProbedStream {
        global_index: 2,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: "h264".to_owned(),
        time_base: Some(Rational::new(1, 600).unwrap()),
        start_time: None,
        duration: Some(time(6_000)),
        disposition: disposition(),
        video: Some(VideoStreamInfo {
            width: 1920,
            height: 1080,
            frame_rate: Some(Rational::new(30, 1).unwrap()),
            cadence: VideoCadence::Constant,
            pixel_format: "yuv420p".to_owned(),
            profile: Some("High".to_owned()),
            level: Some(40),
            sample_aspect_ratio: Rational::new(1, 1).unwrap(),
            rotation_degrees: 0,
        }),
        audio: None,
    }
}

pub fn audio_stream() -> ProbedStream {
    ProbedStream {
        global_index: 5,
        type_index: 0,
        media_type: ProbedStreamType::Audio,
        codec: "aac".to_owned(),
        time_base: Some(Rational::new(1, 48_000).unwrap()),
        start_time: None,
        duration: Some(time(6_000)),
        disposition: disposition(),
        video: None,
        audio: Some(AudioStreamInfo {
            sample_rate: 48_000,
            channels: 2,
            channel_layout: "stereo".to_owned(),
        }),
    }
}

pub fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}

fn disposition() -> StreamDisposition {
    StreamDisposition {
        default: true,
        attached_picture: false,
        timed_thumbnail: false,
    }
}
