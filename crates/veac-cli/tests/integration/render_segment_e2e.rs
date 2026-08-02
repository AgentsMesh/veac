use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use veac_artifact::{ArtifactStore, ExecutionBindings, FullRenderSegmentContract};
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};

use super::support::*;

#[cfg(unix)]
#[test]
fn real_cli_promotes_fresh_delivery_then_reuses_it_with_stream_copy() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let (wrapper, log) = logging_ffmpeg(temp.path());
    let path = path_with(wrapper.parent().unwrap());

    render_with_path(&project, &path).assert().success();
    let first = std::fs::read_to_string(&log).unwrap();
    assert!(first.contains("-c:v libx264"), "{first}");
    assert!(!first.contains("-c copy"), "{first}");
    assert_eq!(video_dimensions(&temp.path().join("render.mp4")), "32x24");

    render_with_path(&project, &path).assert().success();
    let all = std::fs::read_to_string(&log).unwrap();
    let second = all.strip_prefix(&first).unwrap();
    assert!(second.contains("-c copy"), "{second}");
    assert_eq!(video_dimensions(&temp.path().join("render.mp4")), "32x24");
}

#[test]
fn real_cli_require_rejects_semantic_cache_poison_and_prefer_renders_original() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let plan = plan(&project);
    let bindings = ExecutionBindings::from_originals(&plan, &BTreeMap::new()).unwrap();
    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap();
    let contract = FullRenderSegmentContract::new(
        &plan,
        bindings.input_substitution_proof(),
        veac_runtime::workflow::media_artifact_producer(&fingerprint).unwrap(),
    )
    .unwrap();
    let invalid = temp.path().join("wrong-duration.mp4");
    make_video(&invalid);
    ArtifactStore::new(temp.path().join(".veac-artifacts"))
        .put_file(contract.descriptor(), &invalid)
        .unwrap();

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--proxy-policy",
            "original",
            "--render-segment-policy",
            "require",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("RENDER_SEGMENT_POSTFLIGHT_FAILED"));
    assert!(!temp.path().join("render.mp4").exists());

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--proxy-policy",
            "original",
            "--render-segment-policy",
            "prefer",
        ])
        .assert()
        .success();
    assert_eq!(video_dimensions(&temp.path().join("render.mp4")), "32x24");
}

fn plan(path: &Path) -> veac_plan::ResolvedRenderPlan {
    let envelope: veac_ir::ProjectEnvelope =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let output = envelope.project.render_configs[0].id.clone();
    veac_plan::resolve_one(&envelope, &output).unwrap()
}

#[cfg(unix)]
fn render_with_path(project: &Path, path: &std::ffi::OsStr) -> assert_cmd::Command {
    let mut command = veac();
    command.env("PATH", path).args([
        "render",
        project.to_str().unwrap(),
        "--proxy-policy",
        "original",
        "--render-segment-policy",
        "prefer",
    ]);
    command
}

#[cfg(unix)]
fn logging_ffmpeg(directory: &Path) -> (PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;

    let real = ProcessCommand::new("sh")
        .args(["-c", "command -v ffmpeg"])
        .output()
        .unwrap();
    assert!(real.status.success());
    let real = String::from_utf8(real.stdout).unwrap().trim().to_owned();
    let wrapper = directory.join("ffmpeg");
    let log = directory.join("ffmpeg-arguments.log");
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\nexec \"{real}\" \"$@\"\n",
            log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    (wrapper, log)
}

#[cfg(unix)]
fn path_with(directory: &Path) -> std::ffi::OsString {
    let mut paths = vec![directory.to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(paths).unwrap()
}
