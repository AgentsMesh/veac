use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::{
    BackendAction, BackendBundle, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};
use veac_plan::canonical::*;

use super::super::support::{
    bindings, emit_video_command, emit_video_command_for, fixture, resolved,
};

pub(super) fn assert_five_artifact_contract(
    plan: &veac_plan::ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    bundle: &BackendBundle,
) {
    let expected = [
        (
            "dlv_caption",
            BackendProduct::CaptionSidecar,
            BackendOutput::File("/tmp/captions.vtt".into()),
        ),
        (
            "dlv_frames",
            BackendProduct::ImageSequence,
            BackendOutput::ImageSequence {
                pattern: "/tmp/frame-%d.png".into(),
            },
        ),
        (
            "dlv_main",
            BackendProduct::VideoMaster,
            BackendOutput::File("/tmp/output.mp4".into()),
        ),
        (
            "dlv_stem",
            BackendProduct::AudioStem,
            BackendOutput::File("/tmp/master.wav".into()),
        ),
        (
            "dlv_waveform",
            BackendProduct::VideoWaveform,
            BackendOutput::File("/tmp/waveform.png".into()),
        ),
    ];
    let ids = bundle
        .tasks()
        .iter()
        .map(|task| task.deliverable_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, expected.iter().map(|value| value.0).collect());
    for (id, product, output) in expected {
        let task = task(bundle, id);
        assert_eq!(task.phase, BackendPhase::Single);
        assert_eq!(task.product, product);
        assert_eq!(task.output, output);
    }

    let expected_inputs = BTreeMap::from([
        ("dlv_caption", paths(&[])),
        ("dlv_frames", paths(&["/tmp/pin_video.bin"])),
        ("dlv_main", paths(&["/tmp/pin_video.bin"])),
        ("dlv_stem", paths(&[])),
        ("dlv_waveform", paths(&["/tmp/pin_video.bin"])),
    ]);
    let mut physical_union = BTreeSet::new();
    for (id, expected) in expected_inputs {
        let actual = task_inputs(task(bundle, id));
        assert_eq!(actual, expected, "physical inputs for {id}");
        physical_union.extend(actual);
    }
    let protected = bundle
        .protected_resources()
        .iter()
        .map(|resource| resource.path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(protected, physical_union);

    let id = DeliverableId::new("dlv_main").unwrap();
    let selected = emit_video_command_for(plan, bindings, &id).unwrap();
    assert_eq!(selected.output_path, Path::new("/tmp/output.mp4"));
    assert_eq!(selected, emit_video_command(plan, bindings).unwrap());
}

#[test]
fn explicit_video_helper_selects_from_an_ambiguous_group() {
    let (plan, bindings) = two_video_plan();
    let id = DeliverableId::new("dlv_alt").unwrap();
    let command = emit_video_command_for(&plan, &bindings, &id).unwrap();
    assert_eq!(command.output_path, Path::new("/tmp/alternate.mp4"));
}

#[test]
#[should_panic(expected = "test plan must emit exactly one video master")]
fn default_video_helper_rejects_an_ambiguous_group() {
    let (plan, bindings) = two_video_plan();
    let _ = emit_video_command(&plan, &bindings);
}

fn two_video_plan() -> (veac_plan::ResolvedRenderPlan, ExecutionBindings) {
    let mut plan = resolved(&fixture());
    let mut alternate = plan.output.deliverables[0].clone();
    alternate.id = DeliverableId::new("dlv_alt").unwrap();
    alternate.target = DeliverableTarget::File {
        name: "alternate.mp4".to_owned(),
    };
    plan.output.deliverables.push(alternate);
    plan.output
        .deliverables
        .sort_by(|left, right| left.id.cmp(&right.id));
    let local = bindings(&plan);
    (plan, local)
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
