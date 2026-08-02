use super::*;

#[test]
fn output_requires_a_deliverable_and_each_geometry_dimension() {
    let mut empty = sample_project();
    empty.project.render_configs[0].deliverables.clear();
    assert_code(&validation_codes(&empty), "OUTPUT_DELIVERABLE_REQUIRED");

    let mutations: [fn(&mut RenderConfig); 5] = [
        |output: &mut RenderConfig| raster(output).width = 0,
        |output: &mut RenderConfig| raster(output).height = 0,
        |output: &mut RenderConfig| raster(output).frame_rate.numerator = 0,
        |output: &mut RenderConfig| raster(output).frame_rate.denominator = 0,
        |output: &mut RenderConfig| {
            raster(output).frame_rate = Rational {
                numerator: 60,
                denominator: 2,
            }
        },
    ];
    for mutate in mutations {
        let mut project = sample_project();
        mutate(&mut project.project.render_configs[0]);
        assert_code(&validation_codes(&project), "OUTPUT_GEOMETRY");
    }
}

#[test]
fn output_rejects_ffmpeg_integer_overflow_and_subsampled_odd_dimensions() {
    let mutations: [fn(&mut RenderConfig); 7] = [
        |output| raster(output).width = i32::MAX as u32 + 1,
        |output| raster(output).height = i32::MAX as u32 + 1,
        |output| raster(output).width = MAX_DIMENSION + 1,
        |output| (raster(output).width, raster(output).height) = (7_680, 4_320),
        |output| raster(output).frame_rate.numerator = i64::from(i32::MAX) + 1,
        |output| raster(output).frame_rate.denominator = i32::MAX as u32 + 1,
        |output| {
            raster(output).frame_rate = Rational::new(i64::from(MAX_FRAME_RATE) + 1, 1).unwrap()
        },
    ];
    for mutate in mutations {
        let mut project = sample_project();
        mutate(&mut project.project.render_configs[0]);
        assert_code(&validation_codes(&project), "OUTPUT_GEOMETRY");
    }

    let mut project = sample_project();
    project.project.render_configs[0]
        .raster
        .as_mut()
        .unwrap()
        .width |= 1;
    assert_code(&validation_codes(&project), "OUTPUT_VIDEO_SETTINGS");
}

#[test]
fn deliverable_ids_are_valid_unique_global_and_strictly_ordered() {
    let mut invalid = sample_project();
    invalid.project.render_configs[0].deliverables[0].id = serde_json::from_str("\"bad\"").unwrap();
    assert_code(&validation_codes(&invalid), "INVALID_ID");

    let mut duplicate = sample_project();
    let mut second = duplicate.project.render_configs[0].deliverables[0].clone();
    second.target = DeliverableTarget::File {
        name: "second.mp4".to_owned(),
    };
    duplicate.project.render_configs[0]
        .deliverables
        .push(second);
    let codes = validation_codes(&duplicate);
    assert_code(&codes, "DUPLICATE_DELIVERABLE_ID");
    assert_code(&codes, "OUTPUT_DELIVERABLE_ORDER");

    let mut global = sample_project();
    let mut output = global.project.render_configs[0].clone();
    output.id = RenderConfigId::new("out_secondary").unwrap();
    output.deliverables[0].target = DeliverableTarget::File {
        name: "secondary.mp4".to_owned(),
    };
    global.project.render_configs.push(output);
    assert_code(&validation_codes(&global), "DUPLICATE_DELIVERABLE_ID");
}

#[test]
fn distinct_deliverables_must_be_sorted_by_typed_id() {
    let mut project = sample_project();
    let output = &mut project.project.render_configs[0];
    output.deliverables[0].id = DeliverableId::new("dlv_zulu").unwrap();
    let mut earlier = output.deliverables[0].clone();
    earlier.id = DeliverableId::new("dlv_alpha").unwrap();
    earlier.target = DeliverableTarget::File {
        name: "alpha.mp4".to_owned(),
    };
    output.deliverables.push(earlier);

    let codes = validation_codes(&project);
    assert_code(&codes, "OUTPUT_DELIVERABLE_ORDER");
    assert!(!codes.iter().any(|code| code == "DUPLICATE_DELIVERABLE_ID"));
}

#[test]
fn visual_artifacts_require_one_shared_raster() {
    let mut project = sample_project();
    project.project.render_configs[0].raster = None;
    assert_code(&validation_codes(&project), "OUTPUT_RASTER_REQUIRED");
}

#[test]
fn nonvisual_deliveries_reject_unused_raster_and_accept_its_absence() {
    let mut project = sample_project();
    let output = &mut project.project.render_configs[0];
    output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "stem.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS24Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Master,
        }),
    }];
    assert_code(&validation_codes(&project), "OUTPUT_RASTER_UNUSED");
    project.project.render_configs[0].raster = None;
    validate(&project).unwrap();

    let output = &mut project.project.render_configs[0];
    output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_caption").unwrap(),
        target: DeliverableTarget::File {
            name: "captions.srt".into(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Srt,
            track_ids: vec![TrackId::new("trk_captions").unwrap()],
        }),
    }];
    validate(&project).unwrap();
}

#[test]
fn artifact_kind_and_target_kind_must_match_exactly() {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables[0].target = DeliverableTarget::ImageSequence {
        pattern: "frame-%d.mp4".into(),
    };
    assert_code(&validation_codes(&project), "OUTPUT_TARGET_KIND");

    let output = &mut project.project.render_configs[0];
    output.deliverables[0].target = DeliverableTarget::File {
        name: "frame.png".into(),
    };
    output.deliverables[0].kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Png,
        start_number: 1,
    });
    assert_code(&validation_codes(&project), "OUTPUT_TARGET_KIND");
}

fn raster(output: &mut RenderConfig) -> &mut RasterSettings {
    output.raster.as_mut().unwrap()
}
