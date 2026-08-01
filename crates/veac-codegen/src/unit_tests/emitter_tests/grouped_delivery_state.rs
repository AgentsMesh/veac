use std::collections::BTreeSet;
use std::path::PathBuf;

use veac_codegen::emitter::{emit_all, BackendAction, BackendBundle, BackendTask};
use veac_plan::{ResolvedRenderPlan, ResolvedTrack};

use super::support::bindings;

mod support;

#[test]
fn grouped_artifacts_keep_timeline_state_and_physical_inputs_scoped() {
    let plan = support::grouped_plan();
    assert_state(&plan);
    assert_eq!(input_ids(&plan), strings(&["pin_dialogue", "pin_video"]));

    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    let expected = [
        ("dlv_caption", &[][..]),
        ("dlv_frames", &["/tmp/pin_video.bin"] as &[&str]),
        ("dlv_main", &["/tmp/pin_video.bin"] as &[&str]),
        ("dlv_stem", &["/tmp/pin_dialogue.bin"] as &[&str]),
        ("dlv_scope", &["/tmp/pin_video.bin"] as &[&str]),
    ];
    assert_eq!(
        task_ids(&bundle),
        expected.iter().map(|value| value.0.to_owned()).collect()
    );
    let mut physical_union = BTreeSet::new();
    for (id, expected_paths) in expected {
        let actual = task_inputs(task(&bundle, id));
        assert_eq!(actual, paths(expected_paths), "physical inputs for {id}");
        physical_union.extend(actual);
    }
    assert!(matches!(
        task(&bundle, "dlv_caption").action,
        BackendAction::WriteFile { .. }
    ));
    let protected = bundle
        .protected_resources()
        .iter()
        .map(|resource| resource.path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(protected, physical_union);
    assert_eq!(
        protected,
        paths(&["/tmp/pin_dialogue.bin", "/tmp/pin_video.bin"])
    );
}

fn assert_state(plan: &ResolvedRenderPlan) {
    let video = track(plan, "trk_video");
    assert!(video.state.include_in_render);
    assert!(video.state.visual_enabled);
    assert!(!video.state.audio_enabled);

    let dialogue = track(plan, "trk_dialogue");
    assert!(dialogue.state.include_in_render);
    assert!(!dialogue.state.visual_enabled);
    assert!(dialogue.state.audio_enabled);

    let caption = track(plan, "trk_caption");
    assert!(caption.state.include_in_render);
    assert!(!caption.state.visual_enabled);
    assert!(!caption.state.audio_enabled);

    for id in ["trk_music", "trk_disabled"] {
        let omitted = track(plan, id);
        assert!(!omitted.state.include_in_render);
        assert!(!omitted.state.visual_enabled);
        assert!(!omitted.state.audio_enabled);
        assert!(omitted.clips.is_empty());
    }
}

fn track<'a>(plan: &'a ResolvedRenderPlan, id: &str) -> &'a ResolvedTrack {
    plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == id)
        .unwrap()
}

fn input_ids(plan: &ResolvedRenderPlan) -> BTreeSet<String> {
    plan.inputs
        .iter()
        .map(|input| input.id.to_string())
        .collect()
}

fn task_ids(bundle: &BackendBundle) -> BTreeSet<String> {
    bundle
        .tasks()
        .iter()
        .map(|task| task.deliverable_id.to_string())
        .collect()
}

fn task<'a>(bundle: &'a BackendBundle, id: &str) -> &'a BackendTask {
    bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == id)
        .unwrap()
}

fn task_inputs(task: &BackendTask) -> BTreeSet<PathBuf> {
    match &task.action {
        BackendAction::Ffmpeg(command) => command
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect(),
        BackendAction::WriteFile { .. } => BTreeSet::new(),
    }
}

fn paths(values: &[&str]) -> BTreeSet<PathBuf> {
    values.iter().map(PathBuf::from).collect()
}

fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(ToString::to_string).collect()
}
