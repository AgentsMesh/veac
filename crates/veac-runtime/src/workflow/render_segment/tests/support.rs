use veac_artifact::{
    artifact_key, ArtifactRecord, ContentDigest, FullRenderSegmentContract, ProducerFingerprint,
};
use veac_ir::{
    AudioCodec, AudioOutput, AudioStreamInfo, HashAlgorithm, MediaIdentity, MediaProbeSnapshot,
    ProbedStream, ProbedStreamType, Rational, RationalTime, StreamDisposition, StreamSelection,
    VideoStreamInfo,
};

pub(super) fn contract(with_audio: bool) -> FullRenderSegmentContract {
    contract_with(with_audio, |_| {})
}

pub(super) fn contract_with(
    with_audio: bool,
    configure: impl FnOnce(&mut veac_ir::VideoDeliverable),
) -> FullRenderSegmentContract {
    let mut project: veac_ir::ProjectEnvelope = serde_json::from_str(include_str!(
        "../../../../../veac-ir/tests/fixtures/minimal-project.json"
    ))
    .unwrap();
    project.project.materials.clear();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source = veac_ir::ClipSource::Generated {
        generator: veac_ir::Generator::Solid {
            color: veac_ir::Color {
                red: 10,
                green: 20,
                blue: 30,
                alpha: 255,
            },
        },
    };
    clip.source_mapping = None;
    let config = &mut project.project.render_configs[0];
    let deliverable = config
        .deliverables
        .iter_mut()
        .find(|value| matches!(value.kind, veac_ir::DeliverableKind::Video(_)))
        .unwrap();
    let veac_ir::DeliverableKind::Video(video) = &mut deliverable.kind else {
        unreachable!();
    };
    video.audio = with_audio.then_some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    configure(video);
    let extension = match video.container {
        veac_ir::OutputFormat::Mp4 => "mp4",
        veac_ir::OutputFormat::Mov => "mov",
        veac_ir::OutputFormat::Mkv => "mkv",
        veac_ir::OutputFormat::Webm => "webm",
        veac_ir::OutputFormat::Mxf => "mxf",
    };
    deliverable.target = veac_ir::DeliverableTarget::File {
        name: format!("output.{extension}"),
    };
    let output_id = config.id.clone();
    let plan = veac_plan::resolve_one(&project, &output_id).unwrap();
    FullRenderSegmentContract::new(
        &plan,
        ContentDigest::sha256(b"source clocks"),
        ProducerFingerprint {
            name: "ffmpeg".to_owned(),
            version: "fixture".to_owned(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
    )
    .unwrap()
}

pub(super) fn record(contract: &FullRenderSegmentContract) -> ArtifactRecord {
    ArtifactRecord {
        key: artifact_key(contract.descriptor()).unwrap(),
        content: ContentDigest::sha256(b"render segment"),
        size_bytes: 100,
    }
}

pub(super) fn snapshot(
    contract: &FullRenderSegmentContract,
    record: &ArtifactRecord,
) -> MediaProbeSnapshot {
    let profile = contract.media_profile();
    let duration = contract.range().duration;
    let mut streams = vec![ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: "h264".to_owned(),
        time_base: Some(ratio(1, 15_360)),
        start_time: Some(time(0, duration.timescale)),
        duration: Some(duration),
        disposition: disposition(),
        video: Some(VideoStreamInfo {
            width: profile.width(),
            height: profile.height(),
            frame_rate: Some(profile.frame_rate()),
            pixel_format: "yuv420p".to_owned(),
            profile: Some("High".to_owned()),
            level: Some(40),
            sample_aspect_ratio: ratio(1, 1),
            rotation_degrees: 0,
        }),
        audio: None,
    }];
    let selected_audio_stream = profile.video().audio.as_ref().map(|settings| {
        streams.push(ProbedStream {
            global_index: 1,
            type_index: 0,
            media_type: ProbedStreamType::Audio,
            codec: "aac".to_owned(),
            time_base: Some(ratio(1, settings.sample_rate)),
            start_time: Some(time(0, duration.timescale)),
            duration: Some(duration),
            disposition: disposition(),
            video: None,
            audio: Some(AudioStreamInfo {
                sample_rate: settings.sample_rate,
                channels: settings.channels,
                channel_layout: "stereo".to_owned(),
            }),
        });
        StreamSelection {
            global_index: 1,
            type_index: 0,
        }
    });
    MediaProbeSnapshot {
        schema_version: crate::asset::PROBE_SCHEMA_VERSION,
        engine: crate::asset::FIXTURE_PROBE_ENGINE.to_owned(),
        selection_policy: crate::asset::STREAM_SELECTION_POLICY.to_owned(),
        container_format: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
        container_brand: Some("isom".to_owned()),
        observed_identity: identity(record),
        container_duration: Some(duration),
        streams,
        selected_video_stream: Some(StreamSelection {
            global_index: 0,
            type_index: 0,
        }),
        selected_audio_stream,
    }
}

pub(super) fn identity(record: &ArtifactRecord) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: record.content.value.clone(),
    }
}

pub(super) fn extra_stream() -> ProbedStream {
    ProbedStream {
        global_index: 9,
        type_index: 0,
        media_type: ProbedStreamType::Subtitle,
        codec: "subrip".to_owned(),
        time_base: Some(ratio(1, 1_000)),
        start_time: Some(time(0, 1)),
        duration: Some(time(1, 1)),
        disposition: disposition(),
        video: None,
        audio: None,
    }
}

fn disposition() -> StreamDisposition {
    StreamDisposition {
        default: true,
        attached_picture: false,
        timed_thumbnail: false,
    }
}

fn ratio(numerator: i64, denominator: u32) -> Rational {
    Rational::new(numerator, denominator).unwrap()
}

fn time(value: i64, timescale: u32) -> RationalTime {
    RationalTime::new(value, timescale).unwrap()
}
