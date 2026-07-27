use super::*;

#[test]
fn stream_inventory_requires_global_order_typed_ordinals_and_codec_names() {
    for mutate in [
        |probe: &mut MediaProbeSnapshot| probe.streams[1].global_index = 0,
        |probe: &mut MediaProbeSnapshot| probe.streams[1].type_index = 1,
        |probe: &mut MediaProbeSnapshot| probe.streams[0].codec = " ".to_owned(),
    ] {
        let mut project = sample_project();
        mutate(probe_mut(&mut project));
        assert_probe_code(&project, "PROBE_STREAM_ORDER");
    }

    let mut repeated_type = sample_project();
    let probe = probe_mut(&mut repeated_type);
    let mut second_video = probe.streams[0].clone();
    second_video.global_index = 2;
    second_video.type_index = 1;
    probe.streams.push(second_video);
    validate(&repeated_type).unwrap();
}

#[test]
fn video_and_audio_stream_facts_validate_each_required_field() {
    for mutate in [
        |stream: &mut ProbedStream| stream.video.as_mut().unwrap().width = 0,
        |stream: &mut ProbedStream| stream.video.as_mut().unwrap().height = 0,
        |stream: &mut ProbedStream| {
            stream.video.as_mut().unwrap().pixel_format = "YUV420P".to_owned()
        },
        |stream: &mut ProbedStream| {
            stream.video.as_mut().unwrap().profile = Some("bad\nprofile".to_owned())
        },
        |stream: &mut ProbedStream| stream.video.as_mut().unwrap().level = Some(-1),
        |stream: &mut ProbedStream| {
            stream.video.as_mut().unwrap().sample_aspect_ratio = Rational {
                numerator: 0,
                denominator: 1,
            }
        },
        |stream: &mut ProbedStream| {
            stream.audio = Some(AudioStreamInfo {
                sample_rate: 48_000,
                channels: 2,
                channel_layout: "stereo".to_owned(),
            })
        },
    ] {
        let mut project = sample_project();
        mutate(&mut probe_mut(&mut project).streams[0]);
        assert_probe_code(&project, "PROBE_STREAM_FACTS");
    }

    for mutate in [
        |stream: &mut ProbedStream| stream.audio.as_mut().unwrap().sample_rate = 0,
        |stream: &mut ProbedStream| stream.audio.as_mut().unwrap().channels = 0,
        |stream: &mut ProbedStream| stream.audio.as_mut().unwrap().channel_layout = " ".to_owned(),
        |stream: &mut ProbedStream| {
            stream.video = Some(VideoStreamInfo {
                width: 1,
                height: 1,
                frame_rate: Some(Rational::new(30, 1).unwrap()),
                pixel_format: "yuv420p".to_owned(),
                profile: None,
                level: None,
                sample_aspect_ratio: Rational::new(1, 1).unwrap(),
                rotation_degrees: 0,
            })
        },
    ] {
        let mut project = sample_project();
        mutate(&mut probe_mut(&mut project).streams[1]);
        assert_probe_code(&project, "PROBE_STREAM_FACTS");
    }
}

#[test]
fn stream_facts_enforce_decoded_geometry_and_audio_resource_budgets() {
    let video_cases: [fn(&mut VideoStreamInfo); 4] = [
        |video| video.width = MAX_DIMENSION + 1,
        |video| {
            video.width = MAX_DIMENSION;
            video.height = MAX_DIMENSION;
        },
        |video| video.sample_aspect_ratio = Rational::new(16, 1).unwrap(),
        |video| {
            video.width = 3_000;
            video.height = 3_000;
            video.rotation_degrees = 45;
        },
    ];
    for mutate in video_cases {
        let mut project = sample_project();
        mutate(probe_mut(&mut project).streams[0].video.as_mut().unwrap());
        assert_probe_code(&project, "PROBE_STREAM_FACTS");
    }

    let audio_cases: [fn(&mut AudioStreamInfo); 4] = [
        |audio| audio.sample_rate = MAX_INPUT_AUDIO_SAMPLE_RATE + 1,
        |audio| audio.channels = MAX_INPUT_AUDIO_CHANNELS + 1,
        |audio| audio.channel_layout = "stereo\ninvalid".to_owned(),
        |audio| audio.channel_layout = "x".repeat(MAX_CHANNEL_LAYOUT_BYTES + 1),
    ];
    for mutate in audio_cases {
        let mut project = sample_project();
        mutate(probe_mut(&mut project).streams[1].audio.as_mut().unwrap());
        assert_probe_code(&project, "PROBE_STREAM_FACTS");
    }
}

#[test]
fn auxiliary_inventory_types_reject_typed_facts_and_thumbnail_dispositions() {
    let mut valid = sample_project();
    for (index, media_type) in [
        ProbedStreamType::Subtitle,
        ProbedStreamType::Data,
        ProbedStreamType::Attachment,
    ]
    .into_iter()
    .enumerate()
    {
        probe_mut(&mut valid)
            .streams
            .push(aux_stream(index as u32 + 2, media_type));
    }
    validate(&valid).unwrap();

    let mut typed = valid.clone();
    probe_mut(&mut typed).streams[2].video = Some(VideoStreamInfo {
        width: 1,
        height: 1,
        frame_rate: Some(Rational::new(30, 1).unwrap()),
        pixel_format: "yuv420p".to_owned(),
        profile: None,
        level: None,
        sample_aspect_ratio: Rational::new(1, 1).unwrap(),
        rotation_degrees: 0,
    });
    assert_probe_code(&typed, "PROBE_STREAM_FACTS");

    for disposition in [
        StreamDisposition {
            default: false,
            attached_picture: true,
            timed_thumbnail: false,
        },
        StreamDisposition {
            default: false,
            attached_picture: false,
            timed_thumbnail: true,
        },
    ] {
        let mut project = valid.clone();
        probe_mut(&mut project).streams[2].disposition = disposition;
        assert_probe_code(&project, "PROBE_STREAM_FACTS");
    }
}

fn aux_stream(global_index: u32, media_type: ProbedStreamType) -> ProbedStream {
    ProbedStream {
        global_index,
        type_index: 0,
        media_type,
        codec: "fixture".to_owned(),
        time_base: Some(Rational::new(1, 1_000).unwrap()),
        start_time: None,
        duration: None,
        disposition: StreamDisposition {
            default: false,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: None,
        audio: None,
    }
}
