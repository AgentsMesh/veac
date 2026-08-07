use veac_plan::canonical::*;

use super::time;

pub fn identity(fill: char) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: fill.to_string().repeat(64),
    }
}

pub fn video_probe(observed_identity: MediaIdentity) -> MediaProbeSnapshot {
    MediaProbeSnapshot {
        schema_version: MEDIA_PROBE_SCHEMA_VERSION,
        engine: "ffprobe-8.0".to_owned(),
        selection_policy: "veac-stream-selection-v1".to_owned(),
        container_format: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
        container_brand: Some("isom".to_owned()),
        observed_identity,
        container_duration: Some(time(6_000)),
        streams: vec![video_stream(0, 0), audio_stream(1, 0)],
        selected_video_stream: Some(StreamSelection {
            global_index: 0,
            type_index: 0,
        }),
        selected_audio_stream: Some(StreamSelection {
            global_index: 1,
            type_index: 0,
        }),
    }
}

pub fn video_stream(global_index: u32, type_index: u32) -> ProbedStream {
    ProbedStream {
        global_index,
        type_index,
        media_type: ProbedStreamType::Video,
        codec: "h264".to_owned(),
        time_base: Some(Rational::new(1, 600).expect("time base")),
        start_time: Some(time(0)),
        duration: Some(time(6_000)),
        disposition: disposition(),
        video: Some(VideoStreamInfo {
            width: 1920,
            height: 1080,
            frame_rate: Some(Rational::new(30, 1).expect("frame rate")),
            pixel_format: "yuv420p".to_owned(),
            profile: Some("High".to_owned()),
            level: Some(40),
            sample_aspect_ratio: Rational::new(1, 1).expect("ratio"),
            rotation_degrees: 0,
        }),
        audio: None,
    }
}

pub fn audio_stream(global_index: u32, type_index: u32) -> ProbedStream {
    ProbedStream {
        global_index,
        type_index,
        media_type: ProbedStreamType::Audio,
        codec: "aac".to_owned(),
        time_base: Some(Rational::new(1, 48_000).expect("time base")),
        start_time: Some(time(0)),
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

pub fn font_material(id: &str) -> Material {
    Material {
        id: MaterialId::new(id).expect("material id"),
        kind: MaterialKind::Font,
        source: MaterialSource::File {
            uri: "fonts/inter.ttf".to_owned(),
        },
        identity: Some(identity('f')),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        authorship: None,
    }
}

pub fn remote_material(id: &str) -> Material {
    let identity = identity('b');
    Material {
        id: MaterialId::new(id).expect("material id"),
        kind: MaterialKind::Video,
        source: MaterialSource::Remote {
            uri: "https://example.test/video.mp4".to_owned(),
        },
        identity: Some(identity.clone()),
        stream_intent: StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Auto,
        },
        probe: Some(video_probe(identity)),
        authorship: None,
    }
}

pub fn audio_material(id: &str) -> Material {
    let identity = identity('d');
    Material {
        id: MaterialId::new(id).expect("material id"),
        kind: MaterialKind::Audio,
        source: MaterialSource::File {
            uri: "media/audio.wav".to_owned(),
        },
        identity: Some(identity.clone()),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Auto,
        },
        probe: Some(MediaProbeSnapshot {
            schema_version: MEDIA_PROBE_SCHEMA_VERSION,
            engine: "ffprobe-8.0".to_owned(),
            selection_policy: "veac-stream-selection-v1".to_owned(),
            container_format: "wav".to_owned(),
            container_brand: None,
            observed_identity: identity,
            container_duration: Some(time(6_000)),
            streams: vec![audio_stream(3, 0)],
            selected_video_stream: None,
            selected_audio_stream: Some(StreamSelection {
                global_index: 3,
                type_index: 0,
            }),
        }),
        authorship: None,
    }
}

pub fn image_material(id: &str) -> Material {
    let identity = identity('e');
    let mut stream = video_stream(4, 0);
    stream.codec = "png".to_owned();
    stream.duration = None;
    Material {
        id: MaterialId::new(id).expect("material id"),
        kind: MaterialKind::Image,
        source: MaterialSource::File {
            uri: "media/still.png".to_owned(),
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
            container_format: "png_pipe".to_owned(),
            container_brand: None,
            observed_identity: identity,
            container_duration: None,
            streams: vec![stream],
            selected_video_stream: Some(StreamSelection {
                global_index: 4,
                type_index: 0,
            }),
            selected_audio_stream: None,
        }),
        authorship: None,
    }
}

fn disposition() -> StreamDisposition {
    StreamDisposition {
        default: true,
        attached_picture: false,
        timed_thumbnail: false,
    }
}
