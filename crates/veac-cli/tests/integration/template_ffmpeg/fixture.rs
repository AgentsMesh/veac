use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use super::super::support::veac;

pub(super) struct FixtureMedia {
    pub temporal: PathBuf,
    pub portrait: PathBuf,
    pub still: PathBuf,
}

impl FixtureMedia {
    pub(super) fn create(directory: &Path) -> Self {
        let temporal = directory.join("temporal.mp4");
        let portrait = directory.join("portrait.mp4");
        let still = directory.join("still.png");
        temporal_fixture(&temporal);
        portrait_fixture(&portrait);
        image_fixture(&still);
        Self {
            temporal,
            portrait,
            still,
        }
    }
}

pub(super) fn tools_available() -> bool {
    ["ffmpeg", "ffprobe"].iter().all(|tool| {
        ProcessCommand::new(tool)
            .arg("-version")
            .output()
            .is_ok_and(|output| output.status.success())
    })
}

pub(super) fn probe(path: &Path) -> veac_ir::MediaProbeSnapshot {
    let output = veac()
        .args(["probe", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "probing {} failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("normalized probe snapshot")
}

pub(super) fn read_project(path: &Path) -> veac_ir::ProjectEnvelope {
    let input = std::fs::read_to_string(path).unwrap();
    veac_ir::decode_canonical_json(&input).unwrap()
}

pub(super) fn rgb_at(media: &Path, second: f64, x: u32, y: u32) -> [u8; 3] {
    let filter = format!("crop=2:2:{x}:{y},scale=1:1,format=rgb24");
    let output = ProcessCommand::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(second.to_string())
        .arg("-i")
        .arg(media)
        .args(["-vf", &filter, "-frames:v", "1", "-f", "rawvideo", "-"])
        .output()
        .expect("extract rendered pixel");
    assert!(
        output.status.success() && output.stdout.len() >= 3,
        "pixel extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    [output.stdout[0], output.stdout[1], output.stdout[2]]
}

pub(super) fn duration(media: &Path) -> f64 {
    let output = ProcessCommand::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(media)
        .output()
        .expect("inspect rendered duration");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Color {
    Red,
    Green,
    Blue,
}

pub(super) fn assert_pixel(media: &Path, second: f64, x: u32, y: u32, expected: Color) {
    let pixel = rgb_at(media, second, x, y);
    let matches = match expected {
        Color::Red => pixel[0] > 170 && pixel[1] < 90 && pixel[2] < 90,
        Color::Green => pixel[1] > 150 && pixel[0] < 90 && pixel[2] < 90,
        Color::Blue => pixel[2] > 170 && pixel[0] < 90 && pixel[1] < 110,
    };
    assert!(
        matches,
        "unexpected {expected:?} pixel {pixel:?} at {second}s ({x}, {y})"
    );
}

fn temporal_fixture(path: &Path) {
    let mut command = ffmpeg();
    for color in ["red", "lime", "blue"] {
        command.args(["-f", "lavfi", "-i"]);
        command.arg(format!("color=c={color}:s=64x36:r=10:d=1"));
    }
    command.args([
        "-filter_complex",
        "[0:v][1:v][2:v]concat=n=3:v=1:a=0,format=yuv420p[v]",
        "-map",
        "[v]",
        "-c:v",
        "mpeg4",
        "-q:v",
        "1",
    ]);
    finish(command, path);
}

fn portrait_fixture(path: &Path) {
    let mut command = ffmpeg();
    command.args([
        "-f",
        "lavfi",
        "-i",
        "color=c=magenta:s=32x96:r=10:d=1",
        "-vf",
        "drawbox=x=0:y=39:w=10:h=18:color=red:t=fill,drawbox=x=10:y=39:w=12:h=18:color=lime:t=fill,drawbox=x=22:y=39:w=10:h=18:color=blue:t=fill,format=yuv420p",
        "-c:v",
        "mpeg4",
        "-q:v",
        "1",
    ]);
    finish(command, path);
}

fn image_fixture(path: &Path) {
    let mut command = ffmpeg();
    command.args([
        "-f",
        "lavfi",
        "-i",
        "color=c=red:s=48x48:r=1:d=1",
        "-frames:v",
        "1",
    ]);
    finish(command, path);
}

fn ffmpeg() -> ProcessCommand {
    let mut command = ProcessCommand::new("ffmpeg");
    command.args(["-hide_banner", "-loglevel", "error", "-y"]);
    command
}

fn finish(mut command: ProcessCommand, path: &Path) {
    let output = command.arg(path).output().expect("generate fixture media");
    assert!(
        output.status.success(),
        "fixture generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
