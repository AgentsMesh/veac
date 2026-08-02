use crate::{professional_output_tests::project, test_support::sample_project, *};

#[test]
fn canonical_json_and_schema_expose_professional_delivery_types() {
    let mut envelope = project(VideoProfile::DnxHrHqx, PixelFormat::Yuv422p10le);
    envelope.project.render_configs[0]
        .deliverables
        .extend(aux_deliverables());
    envelope.project.render_configs[0]
        .deliverables
        .sort_by(|left, right| left.id.cmp(&right.id));
    let json = canonical_json(&envelope).unwrap();
    let decoded = decode_canonical_json(&json).unwrap();
    assert_eq!(decoded, envelope);
    for value in [
        "dnx_hr",
        "dnx_hr_hqx",
        "pcm_s24_le",
        "pcm_s32_le",
        "tiff",
        "exr",
        "ass",
        "mxf",
    ] {
        assert!(json.contains(value), "missing {value}: {json}");
        assert!(project_json_schema().unwrap().to_string().contains(value));
    }
}

#[test]
fn wav_stems_accept_24_and_32_bit_pcm() {
    for codec in [AudioCodec::PcmS24Le, AudioCodec::PcmS32Le] {
        let mut project = sample_project();
        project.project.render_configs[0].deliverables = vec![stem("dlv_stem", "stem.wav", codec)];
        project.project.render_configs[0].raster = None;
        validate(&project).unwrap();
    }
}

#[test]
fn ass_requires_exact_centisecond_cue_boundaries() {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables = vec![ass()];
    project.project.render_configs[0].raster = None;
    validate(&project).unwrap();

    project.project.sequences[0].tracks[1].clips[0]
        .record_range
        .start = RationalTime::new(1, 600).unwrap();
    let codes: Vec<_> = validate(&project)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|value| value.code)
        .collect();
    assert!(codes.iter().any(|value| value == "OUTPUT_CAPTION_TIME"));
}

fn aux_deliverables() -> Vec<Deliverable> {
    vec![
        ass(),
        image("dlv_exr", "frame-exr-%d.exr", ImageFormat::Exr),
        stem("dlv_pcm24", "pcm24.wav", AudioCodec::PcmS24Le),
        stem("dlv_pcm32", "pcm32.wav", AudioCodec::PcmS32Le),
        image("dlv_tiff", "frame-tiff-%d.tiff", ImageFormat::Tiff),
    ]
}

fn ass() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_ass").unwrap(),
        target: DeliverableTarget::File {
            name: "captions.ass".to_owned(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Ass,
            track_ids: vec![TrackId::new("trk_captions").unwrap()],
        }),
    }
}

fn image(id: &str, file_name: &str, format: ImageFormat) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::ImageSequence {
            pattern: file_name.to_owned(),
        },
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number: 1,
        }),
    }
}

fn stem(id: &str, file_name: &str, codec: AudioCodec) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file_name.to_owned(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec,
                sample_rate: 96_000,
                channels: 2,
            },
            source: AudioMixSource::Master,
        }),
    }
}
