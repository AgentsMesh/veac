use veac_ir::*;

use super::time;

pub fn video(duration: RationalTime, width: u32, height: u32) -> Material {
    media(MaterialKind::Video, Some(duration), width, height)
}

pub fn image(width: u32, height: u32) -> Material {
    media(MaterialKind::Image, None, width, height)
}

pub fn media(
    kind: MaterialKind,
    duration: Option<RationalTime>,
    width: u32,
    height: u32,
) -> Material {
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "b".repeat(64),
    };
    Material {
        id: MaterialId::new("med_slot").unwrap(),
        kind,
        source: MaterialSource::File {
            uri: match kind {
                MaterialKind::Image => "media/replacement.png",
                _ => "media/replacement.mp4",
            }
            .to_owned(),
        },
        identity: Some(identity.clone()),
        stream_intent: StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Disabled,
        },
        probe: Some(MediaProbeSnapshot {
            schema_version: MEDIA_PROBE_SCHEMA_VERSION,
            engine: "ffprobe-8.0".to_owned(),
            selection_policy: "veac-stream-selection-v1".to_owned(),
            container_format: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
            container_brand: Some("isom".to_owned()),
            observed_identity: identity,
            container_duration: duration,
            streams: vec![ProbedStream {
                global_index: 0,
                type_index: 0,
                media_type: ProbedStreamType::Video,
                codec: match kind {
                    MaterialKind::Image => "png",
                    _ => "h264",
                }
                .to_owned(),
                time_base: Some(Rational::new(1, 600).unwrap()),
                start_time: Some(time(0)),
                duration,
                disposition: StreamDisposition {
                    default: true,
                    attached_picture: false,
                    timed_thumbnail: false,
                },
                video: Some(VideoStreamInfo {
                    width,
                    height,
                    frame_rate: Some(Rational::new(30, 1).unwrap()),
                    cadence: VideoCadence::Constant,
                    pixel_format: "yuv420p".to_owned(),
                    profile: Some("High".to_owned()),
                    level: Some(40),
                    sample_aspect_ratio: Rational::new(1, 1).unwrap(),
                    rotation_degrees: 0,
                }),
                audio: None,
            }],
            selected_video_stream: Some(StreamSelection {
                global_index: 0,
                type_index: 0,
            }),
            selected_audio_stream: None,
        }),
        authorship: None,
    }
}

pub fn six_seconds() -> RationalTime {
    time(3600)
}
