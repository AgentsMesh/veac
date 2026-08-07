use super::*;

#[path = "render_budget_tests/hls.rs"]
mod hls;
#[path = "render_budget_tests/structure.rs"]
mod structure;

#[test]
fn canonical_timeline_and_each_delivery_work_domain_are_bounded() {
    let mut project = sample_project();
    let long_ticks = i64::try_from(MAX_TIMELINE_SECONDS).unwrap() * 600 + 1;
    project.project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration = RationalTime::new(long_ticks, 600).unwrap();
    let output = &mut project.project.render_configs[0];
    let raster = output.raster.as_mut().unwrap();
    raster.width = 3_840;
    raster.height = 2_160;
    raster.frame_rate = Rational::new(240, 1).unwrap();
    let DeliverableKind::Video(video) = &mut output.deliverables[0].kind else {
        unreachable!()
    };
    video.audio.as_mut().unwrap().sample_rate = 384_000;
    output.deliverables.extend([
        deliverable(
            "dlv_budget_images",
            "budget-%d.png",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Png,
                start_number: 0,
            }),
        ),
        deliverable(
            "dlv_budget_audio",
            "budget.wav",
            DeliverableKind::AudioStem(AudioStemOutput {
                format: AudioStemFormat::Wav,
                audio: AudioOutput {
                    codec: AudioCodec::PcmS16Le,
                    sample_rate: 384_000,
                    channels: 2,
                },
                source: AudioMixSource::Master,
            }),
        ),
        deliverable(
            "dlv_budget_captions",
            "budget.srt",
            DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::Srt,
                track_ids: vec![TrackId::new("trk_captions").unwrap()],
            }),
        ),
        deliverable(
            "dlv_budget_scope",
            "budget.png",
            DeliverableKind::Scope(ScopeOutput {
                scope: VideoScope::Histogram,
                at: RationalTime::zero(600).unwrap(),
                width: 320,
                height: 180,
                format: ImageFormat::Png,
            }),
        ),
    ]);

    let codes = validation_codes(&project);
    for code in [
        "BUDGET_TIMELINE_DURATION",
        "BUDGET_VIDEO_FRAMES",
        "BUDGET_PIXEL_FRAMES",
        "BUDGET_AUDIO_SAMPLES",
    ] {
        assert_code(&codes, code);
    }
}

#[test]
fn canonical_reverse_and_visual_intermediates_are_bounded() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    sequence.settings.width = 3_840;
    sequence.settings.height = 2_160;
    let clip = &mut sequence.tracks[0].clips[0];
    let SourceTimeMap::Linear {
        rate, direction, ..
    } = &mut clip.source_mapping.as_mut().unwrap().time_map
    else {
        unreachable!()
    };
    *rate = Rational::new(31, 1).unwrap();
    *direction = PlaybackDirection::Reverse;
    let visual = clip.visual.as_mut().unwrap();
    visual.frame = None;
    visual.transform.scale = Animatable::constant(Vec2 { x: 16.0, y: 16.0 });

    let codes = validation_codes(&project);
    assert_code(&codes, "BUDGET_REVERSE_DURATION");
    assert_code(&codes, "BUDGET_VISUAL_INTERMEDIATE_PIXELS");
}

#[test]
fn canonical_placement_intermediate_is_bounded() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    let visual = sequence.tracks[0].clips[0].visual.as_mut().unwrap();
    visual.frame = None;
    visual.transform.scale = Animatable::constant(Vec2 { x: 6.2, y: 6.2 });

    assert_code(
        &validation_codes(&project),
        "BUDGET_VISUAL_INTERMEDIATE_PIXELS",
    );
}

fn deliverable(id: &str, file_name: &str, kind: DeliverableKind) -> Deliverable {
    let target = if matches!(kind, DeliverableKind::ImageSequence(_)) {
        DeliverableTarget::ImageSequence {
            pattern: file_name.to_owned(),
        }
    } else {
        DeliverableTarget::File {
            name: file_name.to_owned(),
        }
    };
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target,
        kind,
    }
}

#[test]
fn structural_usage_counts_effect_curves_source_segments_and_text_animation() {
    let mut project = sample_project();
    let video = &mut project.project.sequences[0].tracks[0].clips[0];
    video.effects[0]
        .effect
        .set_parameter(
            EffectParameter::Brightness,
            EffectParameterValue::Curve(Animatable::Keyframes {
                keyframes: vec![number_key("kf_budget_effect", 0)],
            }),
        )
        .unwrap();
    video.source_mapping.as_mut().unwrap().time_map = SourceTimeMap::Curve {
        segments: vec![SourceTimeSegment {
            record_duration: video.record_range.duration,
            source_start: RationalTime::zero(600).unwrap(),
            source_end: video.record_range.duration,
            interpolation: SourceTimeInterpolation::Linear,
        }],
    };
    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    let ClipSource::Caption { style, .. } = &mut caption.source else {
        unreachable!()
    };
    let mut animation = TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(1.0),
        stagger: RationalTime::zero(600).unwrap(),
    };
    animation.reveal = Animatable::Keyframes {
        keyframes: vec![number_key("kf_budget_text", 0)],
    };
    style.animation = Some(animation);

    let result = validate(&project);
    assert!(result.is_ok(), "{:?}", result.unwrap_err().diagnostics());
}

fn number_key(id: &str, time: i64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::new(time, 600).unwrap(),
        value: 1.0,
        interpolation: Interpolation::Linear,
    }
}
