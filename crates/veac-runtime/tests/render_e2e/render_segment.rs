use tempfile::tempdir;
use veac_artifact::{
    artifact_key, select_full_render_segment, ArtifactRecord, ArtifactStore, ExecutionBindings,
    FullRenderSegmentContract,
};
use veac_codegen::emitter::{emit_all, BackendAction, BackendProduct};
use veac_runtime::executor::{self, FfmpegEnvironment, SystemFfmpeg};
use veac_runtime::workflow::FullRenderSegmentValidator;

use super::support::*;

#[test]
fn exact_full_segment_is_reused_by_real_ffmpeg_and_payload_is_protected() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_background",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_background", color(220, 30, 20), 0, 1_000)],
    ));
    let output_id = canonical.project.render_configs[0].id.clone();
    let plan = veac_plan::resolve_one(&canonical, &output_id).unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = temp.path().join("first.mp4");
    let first_bindings = bindings(&plan, &first);
    let first_bundle = emit_all(&plan, &first_bindings).unwrap();
    let first_execution = executor::execute_bundle(&first_bundle, &store).unwrap();

    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap();
    let contract = FullRenderSegmentContract::new(
        &plan,
        first_bindings.input_substitution_proof(),
        veac_runtime::workflow::media_artifact_producer(&fingerprint).unwrap(),
    )
    .unwrap();
    let expected = segment_record(&contract, &first_execution.tasks[0].output_records[0]);
    let validator = FullRenderSegmentValidator::new(veac_runtime::asset::SystemFfprobe::default());
    validator.validate(&first, &contract, &expected).unwrap();
    store
        .put_file_expected(
            contract.descriptor(),
            &first,
            &expected.content,
            expected.size_bytes,
        )
        .unwrap();
    let artifact = select_full_render_segment(&store, &contract)
        .unwrap()
        .unwrap();
    validator
        .validate(artifact.payload_path(), &contract, artifact.record())
        .unwrap();

    let second = temp.path().join("second.mp4");
    let mut second_bindings = bindings(&plan, &second);
    second_bindings
        .bind_verified_render_segment(&plan, &contract, &artifact)
        .unwrap();
    let second_bundle = emit_all(&plan, &second_bindings).unwrap();
    let command = second_bundle
        .tasks()
        .iter()
        .find_map(|task| match (&task.action, task.product) {
            (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster) => Some(command),
            _ => None,
        })
        .unwrap();
    assert_eq!(command.inputs.len(), 1);
    assert_eq!(command.inputs[0].path, artifact.payload_path());
    assert!(command.filter_graph.is_none());
    assert!(command
        .output_args
        .windows(2)
        .any(|pair| pair == ["-c", "copy"]));
    executor::execute_bundle(&second_bundle, &store).unwrap();
    assert_media_contract(&second, 0, 1.0);
    assert_eq!(rgb_frame(&first, 0.5), rgb_frame(&second, 0.5));

    std::fs::write(artifact.payload_path(), b"corrupt").unwrap();
    let error = executor::execute_bundle(&second_bundle, &store).unwrap_err();
    assert!(error.to_string().contains("identity changed"));
}

fn bindings(plan: &veac_plan::ResolvedRenderPlan, output: &Path) -> ExecutionBindings {
    let mut value = ExecutionBindings::from_originals(plan, &BTreeMap::new()).unwrap();
    value
        .bind_output(plan.output.deliverables[0].id.clone(), output.to_owned())
        .unwrap();
    value
}

fn segment_record(
    contract: &FullRenderSegmentContract,
    rendered: &ArtifactRecord,
) -> ArtifactRecord {
    ArtifactRecord {
        key: artifact_key(contract.descriptor()).unwrap(),
        content: rendered.content.clone(),
        size_bytes: rendered.size_bytes,
    }
}
