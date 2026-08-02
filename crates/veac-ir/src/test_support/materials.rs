use std::collections::BTreeMap;

use crate::*;

use super::time;

pub(super) fn video_material() -> Material {
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "a".repeat(64),
    };
    Material {
        id: MaterialId::new("med_video").unwrap(),
        kind: MaterialKind::Video,
        source: MaterialSource::File {
            uri: "media/video.mp4".to_owned(),
        },
        identity: Some(identity.clone()),
        stream_intent: StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Auto,
        },
        probe: Some(MediaProbeSnapshot {
            schema_version: MEDIA_PROBE_SCHEMA_VERSION,
            engine: "ffprobe-8.0".to_owned(),
            selection_policy: "veac-stream-selection-v1".to_owned(),
            container_format: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
            container_brand: Some("isom".to_owned()),
            observed_identity: identity,
            container_duration: Some(time(6000)),
            streams: vec![video_stream(), audio_stream()],
            selected_video_stream: Some(StreamSelection {
                global_index: 0,
                type_index: 0,
            }),
            selected_audio_stream: Some(StreamSelection {
                global_index: 1,
                type_index: 0,
            }),
        }),
        metadata: BTreeMap::new(),
    }
}

pub(super) fn font_material() -> Material {
    Material {
        id: MaterialId::new("med_font").unwrap(),
        kind: MaterialKind::Font,
        source: MaterialSource::File {
            uri: "fonts/caption.ttf".to_owned(),
        },
        identity: None,
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        metadata: BTreeMap::new(),
    }
}

fn video_stream() -> ProbedStream {
    ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: "h264".to_owned(),
        time_base: Some(Rational::new(1, 600).unwrap()),
        start_time: Some(time(0)),
        duration: Some(time(6000)),
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: Some(VideoStreamInfo {
            width: 1920,
            height: 1080,
            frame_rate: Some(Rational::new(30, 1).unwrap()),
            pixel_format: "yuv420p".to_owned(),
            profile: Some("High".to_owned()),
            level: Some(40),
            sample_aspect_ratio: Rational::new(1, 1).unwrap(),
            rotation_degrees: 0,
        }),
        audio: None,
    }
}

fn audio_stream() -> ProbedStream {
    ProbedStream {
        global_index: 1,
        type_index: 0,
        media_type: ProbedStreamType::Audio,
        codec: "aac".to_owned(),
        time_base: Some(Rational::new(1, 48_000).unwrap()),
        start_time: Some(time(0)),
        duration: Some(time(6000)),
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: None,
        audio: Some(AudioStreamInfo {
            sample_rate: 48_000,
            channels: 2,
            channel_layout: "stereo".to_owned(),
        }),
    }
}
