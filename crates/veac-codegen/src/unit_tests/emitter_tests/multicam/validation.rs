use super::*;

#[test]
fn one_segment_avoids_concat_and_invalid_switch_lists_fail_closed() {
    let mut one = multicam_plan();
    let source = multicam_source_mut(&mut one);
    source.switches.truncate(1);
    source.switches[0].range = TimeRange::new(time(0), time(600)).unwrap();
    let selected = source.switches[0].angle_id.clone();
    source.angles.retain(|angle| angle.id == selected);
    let emitted = emit_all(&one, &bindings(&one)).unwrap();
    let graph = command(&emitted.tasks()[0])
        .filter_graph
        .as_deref()
        .unwrap()
        .to_owned();
    assert!(!graph.contains("concat=n=1"));
    assert!(graph.contains("trim=start=0:duration=1"));
    assert!(graph.contains("atrim=start=0:duration=1"));

    let mut empty = multicam_plan();
    multicam_source_mut(&mut empty).switches.clear();
    assert_invalid(&empty);

    let mut unknown = multicam_plan();
    multicam_source_mut(&mut unknown).switches[0].angle_id =
        MulticamAngleId::new("ang_absent").unwrap();
    assert_invalid(&unknown);

    let mut overflow = multicam_plan();
    multicam_source_mut(&mut overflow).angles[1].source_offset = RationalTime {
        value: 9_007_199_254_740_991,
        timescale: 600,
    };
    assert_invalid(&overflow);

    let mut audio_only = multicam_plan();
    audio_only.sequences[0].tracks[0].state.visual_enabled = false;
    multicam_source_mut(&mut audio_only).switches[0].angle_id =
        MulticamAngleId::new("ang_absent").unwrap();
    assert_invalid(&audio_only);
}

#[test]
fn audio_sync_streams_do_not_force_audio_output_from_a_visual_clip() {
    let mut plan = multicam_plan();
    plan.sequences[0].tracks[0].clips[0].audio = None;

    let emitted = emit_all(&plan, &bindings(&plan)).unwrap();
    let graph = command(&emitted.tasks()[0])
        .filter_graph
        .as_deref()
        .unwrap();
    assert!(!graph.is_empty());
    assert!(!graph.contains("atrim=start=0:duration=1"));
}

#[test]
fn multicam_requires_video_facts_for_each_selected_angle() {
    let mut plan = multicam_plan();
    let input_id = multicam_source_mut(&mut plan).angles[0].input_id.clone();
    plan.inputs
        .iter_mut()
        .find(|input| input.id == input_id)
        .unwrap()
        .video = None;

    assert_invalid(&plan);
}

#[test]
fn multicam_audio_sync_requires_an_audio_stream() {
    let mut plan = multicam_plan();
    multicam_source_mut(&mut plan).angles[0].audio_stream = None;

    assert_invalid(&plan);
}

fn multicam_source_mut(plan: &mut ResolvedRenderPlan) -> &mut ResolvedMulticamSource {
    let ResolvedClipSource::Multicam { source } = &mut plan.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    source
}

fn assert_invalid(plan: &ResolvedRenderPlan) {
    let error = emit_all(plan, &bindings(plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_MULTICAM_INVALID"));
}
