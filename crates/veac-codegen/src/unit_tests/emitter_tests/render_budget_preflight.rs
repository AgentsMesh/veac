use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::ResolvedSourceTimeMap;

use super::support::{bindings, fixture, resolved};

#[test]
fn mutated_plan_timeline_and_delivery_work_fail_before_bundle_creation() {
    let mut plan = resolved(&fixture());
    let long_ticks = i64::try_from(MAX_TIMELINE_SECONDS).unwrap() * 600 + 1;
    plan.sequences[0].duration = RationalTime::new(long_ticks, 600).unwrap();
    let raster = plan.output.raster.as_mut().unwrap();
    raster.width = 3_840;
    raster.height = 2_160;
    raster.frame_rate = Rational::new(240, 1).unwrap();
    let DeliverableKind::Video(video) = &mut plan.output.deliverables[0].kind else {
        unreachable!()
    };
    video.audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 384_000,
        channels: 2,
    });
    plan.output.deliverables.extend([
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
                track_ids: vec![],
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

    let codes = codes(&plan);
    for code in [
        "PLAN_BUDGET_TIMELINE_DURATION",
        "PLAN_BUDGET_VIDEO_FRAMES",
        "PLAN_BUDGET_PIXEL_FRAMES",
        "PLAN_BUDGET_AUDIO_SAMPLES",
    ] {
        assert!(codes.contains(&code), "missing {code}: {codes:?}");
    }
}

#[test]
fn mutated_reverse_and_visual_allocations_fail_before_graph_construction() {
    let mut plan = resolved(&fixture());
    let sequence = &mut plan.sequences[0];
    sequence.settings.width = 3_840;
    sequence.settings.height = 2_160;
    let clip = &mut sequence.tracks[0].clips[0];
    let mapping = clip.source_mapping.as_mut().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        direction,
        ..
    } = &mut mapping.time_map
    else {
        unreachable!()
    };
    source_range_per_repeat.duration = RationalTime::new(18_001, 600).unwrap();
    *direction = PlaybackDirection::Reverse;
    let visual = clip.visual.as_mut().unwrap();
    visual.frame = None;
    visual.transform.scale = Animatable::constant(Vec2 { x: 16.0, y: 16.0 });

    let codes = codes(&plan);
    assert!(codes.contains(&"PLAN_BUDGET_REVERSE_DURATION"));
    assert!(codes.contains(&"PLAN_BUDGET_VISUAL_INTERMEDIATE_PIXELS"));
}

#[test]
fn placement_allocation_is_bounded_before_graph_construction() {
    let mut plan = resolved(&fixture());
    let sequence = &mut plan.sequences[0];
    let visual = sequence.tracks[0].clips[0].visual.as_mut().unwrap();
    visual.frame = None;
    visual.transform.scale = Animatable::constant(Vec2 { x: 6.2, y: 6.2 });

    assert!(codes(&plan).contains(&"PLAN_BUDGET_VISUAL_INTERMEDIATE_PIXELS"));
}

#[test]
fn large_shadow_offset_is_bounded_before_graph_construction() {
    let mut plan = resolved(&fixture());
    let visual = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap();
    visual.card = Some(CardStyle {
        corner_radius_pixels: 0.0,
        shadow: Some(Shadow {
            blur_pixels: 0.0,
            opacity: 1.0,
            offset: Vec2 {
                x: 100_000.0,
                y: 100_000.0,
            },
            color: Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        }),
    });

    assert!(codes(&plan).contains(&"PLAN_BUDGET_VISUAL_INTERMEDIATE_PIXELS"));
}

#[test]
fn mutated_plan_structure_is_bounded_before_reference_walks() {
    let mut plan = resolved(&fixture());
    let sequence = &mut plan.sequences[0];
    let mut template = sequence.tracks[0].clone();
    template.clips.clear();
    template.transitions.clear();
    for index in sequence.tracks.len()..=MAX_TOTAL_TRACKS as usize {
        let mut track = template.clone();
        track.id = TrackId::new(format!("trk_budget_{index}")).unwrap();
        track.source_order = index as u32;
        track.order = index as i32;
        sequence.tracks.push(track);
    }

    assert!(codes(&plan).contains(&"PLAN_BUDGET_TRACKS"));
}

fn deliverable(id: &str, file_name: &str, kind: DeliverableKind) -> Deliverable {
    let target = match &kind {
        DeliverableKind::ImageSequence(_) => DeliverableTarget::ImageSequence {
            pattern: file_name.to_owned(),
        },
        _ => DeliverableTarget::File {
            name: file_name.to_owned(),
        },
    };
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target,
        kind,
    }
}

fn codes(plan: &veac_plan::ResolvedRenderPlan) -> Vec<&'static str> {
    emit_all(plan, &bindings(plan))
        .unwrap_err()
        .diagnostics()
        .iter()
        .map(|value| value.code)
        .collect()
}
