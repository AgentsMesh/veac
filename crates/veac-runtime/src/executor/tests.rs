use veac_artifact::{ArtifactStore, ExecutionBindings};
use veac_codegen::emitter::{
    BackendAction, BackendBundle, BackendCommand, BackendOutput, BackendPhase, BackendProduct,
    BackendTask,
};
use veac_plan::ResolvedRenderPlan;

use super::*;

#[path = "tests/bundle_tests.rs"]
mod bundle_tests;
#[path = "tests/contract_hardening_tests.rs"]
mod contract_hardening_tests;
#[path = "tests/deadline_tests.rs"]
mod deadline_tests;
#[path = "tests/failure_tests.rs"]
mod failure_tests;
#[path = "tests/passlog_snapshot_tests.rs"]
mod passlog_snapshot_tests;
#[path = "tests/path_contract_tests.rs"]
mod path_contract_tests;
#[path = "tests/requirement_tests.rs"]
mod requirement_tests;
#[cfg(unix)]
#[path = "tests/resource_alias_tests.rs"]
mod resource_alias_tests;
#[path = "tests/resource_identity_tests.rs"]
mod resource_identity_tests;
#[path = "tests/resume_tests.rs"]
mod resume_tests;
#[path = "tests/snapshot_tests.rs"]
mod snapshot_tests;
#[path = "tests/substitution_identity_tests.rs"]
mod substitution_identity_tests;
#[path = "tests/support.rs"]
pub(in crate::executor) mod support;

#[test]
fn sealed_bundle_embeds_the_resolved_plan_identity() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("master.mp4");
    let (bundle, plan) = sealed_generated_bundle(output.clone());
    assert_eq!(
        bundle.plan_identity().value,
        veac_plan::plan_hash(&plan).unwrap()
    );
    assert!(!output.exists());
}

#[test]
fn reports_installed_ffmpeg_version() {
    let version = check_ffmpeg().expect("FFmpeg is required by the test environment");
    assert!(version.starts_with("ffmpeg version"));
}

#[test]
fn system_environment_reports_queries_and_missing_binary() {
    let environment = SystemFfmpeg::default();
    assert!(environment
        .fingerprint()
        .expect("fingerprint succeeds")
        .version
        .starts_with("ffmpeg version"));
    assert!(!environment
        .encoders()
        .expect("encoder query succeeds")
        .is_empty());
    assert!(environment
        .muxers()
        .expect("muxer query succeeds")
        .contains("mxf"));

    let missing = SystemFfmpeg::new("veac-no-such-ffmpeg")
        .fingerprint()
        .expect_err("missing executable must fail before execution");
    assert!(missing.message.contains("failed to run FFmpeg"));
}

#[test]
fn executes_a_real_ffmpeg_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("master.mp4");
    let (bundle, _) = sealed_generated_bundle(output.clone());
    execute_bundle(&bundle, &ArtifactStore::new(temp.path().join("store")))
        .expect("backend bundle renders");
    assert!(output.exists());
}

#[test]
fn guarded_real_bundle_surfaces_ffmpeg_failure() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("master.mp4");
    let mut task = real_video_task(output);
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command
        .output_args
        .insert(0, "-definitely-not-a-real-option".into());
    let error = BundleExecutor::new(SystemFfmpeg::default())
        .execute_runtime(
            &support::bundle(vec![task]),
            &ArtifactStore::new(temp.path().join("store")),
        )
        .expect_err("invalid FFmpeg option must fail through the guarded bundle path");
    assert!(error.message.contains("FFmpeg render failed"));
}

fn real_video_task(output: std::path::PathBuf) -> BackendTask {
    let command = BackendCommand {
        inputs: vec![],
        filter_graph: Some("color=c=black:s=32x32:d=0.1[outv]".into()),
        filter_contract: None,
        maps: vec!["[outv]".into()],
        output_args: vec![
            "-frames:v".into(),
            "1".into(),
            "-c:v".into(),
            "libx264".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-f".into(),
            "mp4".into(),
        ],
        output_path: output.clone(),
    };
    BackendTask {
        deliverable_id: support::deliverable_id("master"),
        phase: BackendPhase::Single,
        product: BackendProduct::VideoMaster,
        output: BackendOutput::File(output.clone()),
        action: BackendAction::Ffmpeg(command),
    }
}

fn sealed_generated_bundle(output: std::path::PathBuf) -> (BackendBundle, ResolvedRenderPlan) {
    let mut project: veac_ir::ProjectEnvelope = serde_json::from_str(include_str!(
        "../../../veac-ir/tests/fixtures/minimal-project.json"
    ))
    .unwrap();
    project.project.materials.clear();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source = veac_ir::ClipSource::Generated {
        generator: veac_ir::Generator::Solid {
            color: veac_ir::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        },
    };
    clip.source_mapping = None;
    let config = project.project.render_configs[0].id.clone();
    let plan = veac_plan::resolve_one(&project, &config).unwrap();
    let mut bindings = ExecutionBindings::default();
    bindings
        .bind_output(plan.output.deliverables[0].id.clone(), output)
        .unwrap();
    let bundle = veac_codegen::emitter::emit_all(&plan, &bindings).unwrap();
    (bundle, plan)
}
