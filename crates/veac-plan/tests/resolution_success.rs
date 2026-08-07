mod support;

use support::*;
use veac_plan::{
    canonical::*, resolve, resolve_one, ResolvedSourceTimeMap, CURRENT_RENDER_PLAN_VERSION,
};

#[test]
fn resolves_complete_snapshot_into_owned_exact_plan() {
    let mut project = project();
    let mut authored = visual_properties();
    authored.transform.position = Animatable::constant(Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    });
    project.project.sequences[0].tracks[0].clips[0].visual = Some(authored);
    let transform = &mut project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .transform;
    transform.flip_horizontal = true;
    transform.flip_vertical = true;
    let plans = resolve(&project, None).expect("snapshot resolves");
    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    assert_eq!(plan.header.schema_version, CURRENT_RENDER_PLAN_VERSION);
    assert_eq!(plan.header.source.project_id.as_str(), "prj_fixture");
    assert_eq!(plan.header.source.semantic_hash.len(), 64);
    assert_eq!(plan.header.source.snapshot_hash.len(), 64);
    assert_eq!(plan.output.render_config_id.as_str(), "out_main");
    assert_eq!(plan.inputs.len(), 1);
    assert_eq!(
        plan.inputs[0]
            .video
            .as_ref()
            .unwrap()
            .selection
            .global_index,
        0
    );
    assert_eq!(
        plan.inputs[0]
            .audio
            .as_ref()
            .unwrap()
            .selection
            .global_index,
        1
    );
    let track = &plan.sequences[0].tracks[0];
    assert_eq!(track.placement_mode, PlacementMode::Magnetic);
    assert!(track.state.visual_enabled);
    assert!(!track.state.audio_enabled);
    assert!(track.clips[0].visual.is_some());
    let visual = track.clips[0].visual.as_ref().unwrap();
    let Animatable::Constant { value: offset } = &visual.transform.position else {
        panic!("default position must be constant");
    };
    assert_eq!(offset.x.value, 0.0);
    assert_eq!(offset.x.unit, LengthUnit::Pixels);
    assert!(visual.transform.flip_horizontal);
    assert!(visual.transform.flip_vertical);
    let mapping = track.clips[0].source_mapping.as_ref().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = mapping.time_map
    else {
        panic!("expected linear mapping");
    };
    assert_eq!(source_range_per_repeat, range(0, 600));
    assert_eq!(plan.sequences[0].duration, time(600));
}

#[test]
fn selected_config_and_all_configs_have_stable_id_order() {
    let mut project = project();
    let mut early = project.project.render_configs[0].clone();
    early.id = RenderConfigId::new("out_a").unwrap();
    early.deliverables[0].id = DeliverableId::new("dlv_a").unwrap();
    early.deliverables[0].target = DeliverableTarget::File {
        name: "a.mp4".to_owned(),
    };
    project.project.render_configs.push(early);
    let plans = resolve(&project, None).unwrap();
    let ids: Vec<_> = plans
        .iter()
        .map(|plan| plan.output.render_config_id.as_str())
        .collect();
    assert_eq!(ids, ["out_a", "out_main"]);
    let selected = resolve_one(&project, &RenderConfigId::new("out_main").unwrap()).unwrap();
    assert_eq!(selected.output.id.as_str(), "pout_main");
}

#[test]
fn nested_sequences_are_dependency_first_and_tracks_are_ordered() {
    let mut project = project();
    let nested = sequence(
        "seq_nested",
        vec![track(
            "trk_nested",
            TrackKind::Visual,
            0,
            vec![generated_clip("itm_nested", Generator::Transparent, 0)],
        )],
    );
    project.project.sequences.push(nested);
    let nested_clip = Clip {
        id: ItemId::new("itm_precomp").unwrap(),
        enabled: true,
        record_range: range(0, 300),
        source: ClipSource::Sequence {
            sequence_id: SequenceId::new("seq_nested").unwrap(),
        },
        source_mapping: Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap())),
        visual: None,
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    };
    project.project.sequences[0].tracks.insert(
        0,
        track("trk_precomp", TrackKind::Visual, 10, vec![nested_clip]),
    );
    let plan = resolve(&project, None).unwrap().remove(0);
    let sequences: Vec<_> = plan
        .sequences
        .iter()
        .map(|sequence| sequence.id.as_str())
        .collect();
    assert_eq!(sequences, ["seq_nested", "seq_main"]);
    let orders: Vec<_> = plan.sequences[1]
        .tracks
        .iter()
        .map(|track| track.order)
        .collect();
    assert_eq!(orders, [0, 10]);
}

#[test]
fn solo_filter_prevents_unused_remote_material_resolution() {
    let mut project = project();
    project
        .project
        .materials
        .push(remote_material("med_remote"));
    project.project.sequences[0].tracks[0].state.solo = true;
    project.project.sequences[0].tracks.push(track(
        "trk_remote",
        TrackKind::Visual,
        20,
        vec![media_clip("itm_remote", "med_remote", 0)],
    ));
    let plan = resolve(&project, None).unwrap().remove(0);
    assert_eq!(plan.inputs.len(), 1);
    let remote = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_remote")
        .unwrap();
    assert!(!remote.state.include_in_render);
    assert!(remote.clips.is_empty());
}
