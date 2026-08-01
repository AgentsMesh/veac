use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::{
    ResolvedClipSource, ResolvedMulticamAngle, ResolvedMulticamSource, ResolvedRenderPlan,
};

use super::support::{bindings, fixture, resolved, time};

#[test]
fn media_and_freeze_reject_stream_indexes_from_different_facts() {
    let mut media = resolved(&fixture());
    let ResolvedClipSource::Media {
        video_stream: Some(selection),
        ..
    } = &mut clip(&mut media).source
    else {
        unreachable!()
    };
    selection.global_index += 1;
    assert_invalid(&media);

    let mut freeze = resolved(&fixture());
    let input_id = freeze.inputs[0].id.clone();
    let mut selection = freeze.inputs[0].video.as_ref().unwrap().selection;
    selection.type_index += 1;
    clip(&mut freeze).source = ResolvedClipSource::FreezeFrame {
        input_id,
        video_stream: selection,
        source_time: time(0),
    };
    assert_invalid(&freeze);
}

#[test]
fn audio_selection_must_match_the_selected_audio_facts() {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    project.project.sequences[0].tracks[0].clips[0].audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    let mut plan = resolved(&project);
    let ResolvedClipSource::Media {
        audio_stream: Some(selection),
        ..
    } = &mut clip(&mut plan).source
    else {
        unreachable!()
    };
    selection.global_index += 1;
    assert_invalid(&plan);

    let mut missing = resolved(&project);
    let ResolvedClipSource::Media { audio_stream, .. } = &mut clip(&mut missing).source else {
        unreachable!()
    };
    *audio_stream = None;
    assert_invalid(&missing);
}

#[test]
fn multicam_angles_cannot_mix_one_stream_index_with_another_streams_facts() {
    let mut plan = resolved(&fixture());
    let input_id = plan.inputs[0].id.clone();
    let mut selection = plan.inputs[0].video.as_ref().unwrap().selection;
    selection.global_index += 1;
    clip(&mut plan).source = ResolvedClipSource::Multicam {
        source: ResolvedMulticamSource {
            group_id: MulticamGroupId::new("mcg_preflight").unwrap(),
            sync: MulticamSync {
                basis: MulticamSyncBasis::Manual,
                reference_angle_id: MulticamAngleId::new("ang_preflight").unwrap(),
            },
            angles: vec![ResolvedMulticamAngle {
                id: MulticamAngleId::new("ang_preflight").unwrap(),
                input_id,
                video_stream: selection,
                audio_stream: None,
                source_offset: time(0),
            }],
            switches: vec![MulticamSwitch {
                angle_id: MulticamAngleId::new("ang_preflight").unwrap(),
                range: TimeRange::new(time(0), time(600)).unwrap(),
            }],
        },
    };
    assert_invalid(&plan);
}

fn clip(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}

fn assert_invalid(plan: &ResolvedRenderPlan) {
    let error = emit_all(plan, &bindings(plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|item| item.code == "PLAN_INPUT_STREAM_INVALID"));
}
