use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

mod executable;
pub(crate) use executable::{EXECUTABLE_SOURCE, MEDIA_SOURCE};

pub(crate) use assert_cmd::Command;
pub(crate) use predicates::prelude::*;
pub(crate) use tempfile::{tempdir, TempDir};

pub(crate) const GENERATED_SOURCE: &str = EXECUTABLE_SOURCE;

pub(crate) fn veac() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("veac"))
}

pub(crate) fn source_file(temp: &TempDir, source: &str) -> PathBuf {
    let path = temp.path().join("main.veac");
    std::fs::write(&path, source).unwrap();
    path
}

pub(crate) fn compile_ir(temp: &TempDir, source: &str) -> PathBuf {
    let source = if source == MEDIA_SOURCE && temp.path().join("clip.mp4").is_file() {
        let identity = veac_runtime::asset::sha256_identity(&temp.path().join("clip.mp4")).unwrap();
        source.replace(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &identity.digest,
        )
    } else {
        source.to_owned()
    };
    let source_path = source_file(temp, &source);
    let project = temp.path().join("project.json");
    let envelope = veac_lang::program::build_path(&source_path)
        .unwrap()
        .envelope()
        .clone();
    std::fs::write(&project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
    project
}

pub(crate) fn make_video(path: &Path) {
    let status = ProcessCommand::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=32x24:d=0.4:r=10",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(path)
        .status()
        .expect("FFmpeg is required for CLI E2E tests");
    assert!(status.success());
}

pub(crate) fn video_dimensions(path: &Path) -> String {
    let output = ProcessCommand::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0:s=x",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[cfg(unix)]
pub(crate) fn fake_audio_ffprobe(root: &Path, index: u32) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join("ffprobe-fixture.sh");
    let json = format!(
        r#"{{"streams":[{{"index":{index},"codec_type":"audio","codec_name":"pcm_s16le","time_base":"1/48000","start_time":"0.000000","duration":"1.000000","sample_rate":"48000","channels":2,"channel_layout":"stereo"}}],"format":{{"format_name":"wav","duration":"1.000000"}}}}"#
    );
    std::fs::write(
        &path,
        format!("#!/bin/sh\nif [ \"$1\" = -version ]; then printf 'ffprobe version cli-e2e-fixture-v1\\n'; exit 0; fi\nprintf '%s' '{json}'\n"),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}
