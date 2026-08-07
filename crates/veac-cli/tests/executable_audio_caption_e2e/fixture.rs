use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;

pub const FONT_SHA: &str = "64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774";
pub const TONE_SHA: &str = "5c3aaa006e4c341feddf589f9eb0ba3b215c60353e491abf8f06f4bee4478cfc";
const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/executable-audio-caption/main.veac"
));

pub struct Paths {
    pub project: PathBuf,
    pub rendered: PathBuf,
    tone: PathBuf,
    root: PathBuf,
}

pub fn prepare(root: &Path) -> Paths {
    let example =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/executable-audio-caption");
    let assets = root.join("assets");
    let tone = assets.join("tone.wav");
    let font = assets.join("veac-example-zh.ttf");
    std::fs::create_dir(&assets).unwrap();
    std::fs::copy(example.join("assets/tone.wav"), &tone).unwrap();
    std::fs::copy(example.join("assets/veac-example-zh.ttf"), &font).unwrap();
    assert_eq!(identity(&tone), TONE_SHA);
    assert_eq!(identity(&font), FONT_SHA);
    std::fs::write(root.join("main.veac"), SOURCE).unwrap();
    std::fs::copy(example.join("showcase.veac"), root.join("showcase.veac")).unwrap();
    let rendered = root.join("rendered");
    std::fs::create_dir(&rendered).unwrap();
    Paths {
        project: root.join("project.json"),
        rendered,
        tone,
        root: root.to_owned(),
    }
}

pub fn build(paths: &Paths) -> veac_ir::ProjectEnvelope {
    veac()
        .args([
            "build",
            paths.root.join("main.veac").to_str().unwrap(),
            "--emit-ir",
        ])
        .arg(&paths.project)
        .assert()
        .success();
    let json = std::fs::read_to_string(&paths.project).unwrap();
    veac_ir::decode_canonical_json(&json).unwrap()
}

pub fn plan(paths: &Paths) -> veac_plan::ResolvedRenderPlan {
    let output = veac().args(["plan"]).arg(&paths.project).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub fn render(paths: &Paths) {
    veac()
        .args(["render"])
        .arg(&paths.project)
        .args(["--destination"])
        .arg(&paths.rendered)
        .assert()
        .success();
}

pub fn assert_stale_identity_stops_before_ffmpeg(paths: &Paths) {
    std::fs::OpenOptions::new()
        .append(true)
        .open(&paths.tone)
        .unwrap()
        .write_all(b"stale")
        .unwrap();
    let destination = paths.root.join("stale-rendered");
    std::fs::create_dir(&destination).unwrap();
    let (bin, marker) = fake_ffmpeg(&paths.root);
    let output = veac()
        .env("PATH", path_with(&bin))
        .env("VEAC_FFMPEG_MARKER", &marker)
        .args(["render"])
        .arg(&paths.project)
        .args(["--destination"])
        .arg(&destination)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("MATERIAL_IDENTITY_MISMATCH"));
    assert!(!marker.exists(), "FFmpeg ran before identity validation");
    assert!(!destination.join("preview.mp4").exists());
}

fn identity(path: &Path) -> String {
    veac_runtime::asset::sha256_identity(path).unwrap().digest
}

fn veac() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("veac"))
}

fn path_with(first: &Path) -> OsString {
    let current = std::env::var_os("PATH").unwrap_or_default();
    std::env::join_paths(std::iter::once(first.to_owned()).chain(std::env::split_paths(&current)))
        .unwrap()
}

#[cfg(unix)]
fn fake_ffmpeg(root: &Path) -> (PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;

    let bin = root.join("fake-bin");
    let marker = root.join("ffmpeg-called");
    std::fs::create_dir(&bin).unwrap();
    let executable = bin.join("ffmpeg");
    std::fs::write(
        &executable,
        "#!/bin/sh\n: > \"$VEAC_FFMPEG_MARKER\"\nexit 97\n",
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    (bin, marker)
}

#[cfg(not(unix))]
fn fake_ffmpeg(root: &Path) -> (PathBuf, PathBuf) {
    (root.to_owned(), root.join("ffmpeg-called"))
}
