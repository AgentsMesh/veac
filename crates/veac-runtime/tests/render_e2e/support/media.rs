use std::process::Command;

use super::*;

pub(crate) fn material(
    id: &str,
    kind: MaterialKind,
    video: StreamChoice,
    audio: StreamChoice,
) -> Material {
    Material {
        id: MaterialId::new(id).unwrap(),
        kind,
        source: MaterialSource::File {
            uri: format!("fixtures/{id}"),
        },
        identity: None,
        stream_intent: StreamIntent { video, audio },
        probe: None,
        authorship: None,
    }
}

pub(crate) fn attached_picture_fixture(dir: &Path) -> PathBuf {
    let cover = dir.join("cover.png");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=24x24",
            "-frames:v",
            "1",
            "-update",
            "1",
        ])
        .arg(&cover));
    let output = dir.join("attached.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=64x36:r=10:d=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=330:sample_rate=48000:duration=2",
            "-i",
        ])
        .arg(&cover)
        .args([
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-map",
            "2:v:0",
            "-c:v:0",
            "libx264",
            "-pix_fmt:v:0",
            "yuv420p",
            "-c:a:0",
            "aac",
            "-c:v:1",
            "mjpeg",
            "-disposition:v:0",
            "default",
            "-disposition:v:1",
            "attached_pic",
            "-t",
            "2",
        ])
        .arg(&output));
    output
}

pub(crate) fn tone_fixture(dir: &Path, name: &str, frequency: u32) -> PathBuf {
    let output = dir.join(format!("{name}.m4a"));
    let source = format!("sine=frequency={frequency}:sample_rate=48000:duration=1");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &source,
            "-c:a",
            "aac",
            "-ac",
            "1",
        ])
        .arg(&output));
    output
}

pub(crate) fn color_video_fixture(dir: &Path, name: &str, color: &str) -> PathBuf {
    let output = dir.join(format!("{name}.mp4"));
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("color=c={color}:s={WIDTH}x{HEIGHT}:r={FPS}:d=2"),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output));
    output
}

pub(crate) fn retime_fixture(dir: &Path) -> PathBuf {
    let output = dir.join("retime.mp4");
    let mut command = Command::new("ffmpeg");
    command.args(["-hide_banner", "-loglevel", "error", "-y"]);
    for color in ["red", "green", "blue", "yellow"] {
        command.args([
            "-f",
            "lavfi",
            "-i",
            &format!("color=c={color}:s=96x54:r=10:d=1"),
        ]);
    }
    command.args([
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=4",
        "-filter_complex",
        "[0:v][1:v][2:v][3:v]concat=n=4:v=1:a=0[v]",
        "-map",
        "[v]",
        "-map",
        "4:a:0",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-ac",
        "1",
    ]);
    run(command.arg(&output));
    output
}

pub(crate) fn font_fixture() -> PathBuf {
    [
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
    .expect("a supported system test font")
}

fn run(command: &mut Command) {
    let output = command.output().expect("start FFmpeg fixture command");
    assert!(
        output.status.success(),
        "FFmpeg fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
