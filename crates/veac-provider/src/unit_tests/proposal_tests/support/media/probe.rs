use veac_ir::*;

pub(super) fn intent(kind: MaterialKind) -> StreamIntent {
    StreamIntent {
        video: if kind == MaterialKind::Video {
            StreamChoice::Auto
        } else {
            StreamChoice::Disabled
        },
        audio: if kind == MaterialKind::Audio {
            StreamChoice::Auto
        } else {
            StreamChoice::Disabled
        },
    }
}

pub(super) fn probe(
    kind: MaterialKind,
    identity: MediaIdentity,
    duration: RationalTime,
) -> MediaProbeSnapshot {
    let media_type = if kind == MaterialKind::Audio {
        ProbedStreamType::Audio
    } else {
        ProbedStreamType::Video
    };
    let selection = StreamSelection {
        global_index: 0,
        type_index: 0,
    };
    MediaProbeSnapshot {
        schema_version: MEDIA_PROBE_SCHEMA_VERSION,
        engine: "fixture-probe".into(),
        selection_policy: "fixture-v1".into(),
        container_format: if kind == MaterialKind::Audio {
            "wav".into()
        } else {
            "mov,mp4,m4a,3gp,3g2,mj2".into()
        },
        container_brand: (kind != MaterialKind::Audio).then(|| "isom".into()),
        observed_identity: identity,
        container_duration: Some(duration),
        streams: vec![ProbedStream {
            global_index: 0,
            type_index: 0,
            media_type,
            codec: "fixture".into(),
            time_base: Some(Rational::new(1, 600).unwrap()),
            start_time: Some(RationalTime::zero(duration.timescale).unwrap()),
            duration: Some(duration),
            disposition: StreamDisposition {
                default: true,
                attached_picture: false,
                timed_thumbnail: false,
            },
            video: (kind == MaterialKind::Video).then_some(VideoStreamInfo {
                width: 1920,
                height: 1080,
                frame_rate: Some(Rational::new(30, 1).unwrap()),
                pixel_format: "yuv420p".to_owned(),
                profile: Some("High".to_owned()),
                level: Some(40),
                sample_aspect_ratio: Rational::new(1, 1).unwrap(),
                rotation_degrees: 0,
            }),
            audio: (kind == MaterialKind::Audio).then_some(AudioStreamInfo {
                sample_rate: 48_000,
                channels: 2,
                channel_layout: "stereo".into(),
            }),
        }],
        selected_video_stream: (kind == MaterialKind::Video).then_some(selection),
        selected_audio_stream: (kind == MaterialKind::Audio).then_some(selection),
    }
}
