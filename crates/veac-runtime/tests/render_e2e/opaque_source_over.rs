use std::{fs, process::Command};

use tempfile::tempdir;
use veac_codegen::emitter::{BackendAction, BackendProduct};

use super::support::*;

const FRAME_COUNT: usize = 20;
const PIXELS: usize = (WIDTH * HEIGHT) as usize;
const FRAME_BYTES: usize = PIXELS * 4 * size_of::<u16>();
const BASE: [u8; 4] = [32, 96, 160, 255];
const MASKED: [u8; 4] = [224, 64, 16, 64];
const ZERO_ALPHA: [u8; 4] = [255, 255, 255, 0];

#[test]
fn opaque_source_over_preserves_exact_rgb_alpha_and_half_open_windows() {
    let temp = tempdir().unwrap();
    let base_path = rgba_fixture(temp.path(), "base", BASE, 2.0);
    let masked_path = rgba_fixture(temp.path(), "masked", MASKED, 0.6);
    let zero_path = rgba_fixture(temp.path(), "zero", ZERO_ALPHA, 0.3);
    let mut canonical = project(false);
    canonical.project.materials.extend([
        video_material("med_base"),
        video_material("med_masked"),
        video_material("med_zero"),
    ]);
    canonical.project.sequences[0].tracks.extend([
        media_track("base", 0, "med_base", 0, 2_000),
        media_track("masked", 1, "med_masked", 500, 600),
        media_track("zero", 2, "med_zero", 1_300, 300),
    ]);
    let assets = BTreeMap::from([
        ("med_base".to_owned(), base_path),
        ("med_masked".to_owned(), masked_path),
        ("med_zero".to_owned(), zero_path),
    ]);
    let raw = temp.path().join("opaque-source-over.gbrap16le");
    let (graph, frames) = render_raw(canonical, &assets, &raw);

    assert_eq!(graph.matches("maskedmerge=planes=7:enable=").count(), 3);
    for interval in [
        "gte(t,0)*lt(t,2)",
        "gte(t,0.5)*lt(t,1.1)",
        "gte(t,1.3)*lt(t,1.6)",
    ] {
        assert!(graph.contains(interval), "missing {interval}: {graph}");
    }
    assert!(!graph.contains("unpremultiply=planes=7"), "{graph}");

    let base = expanded_rgba(BASE);
    let inside = masked_rgb(base, expanded_rgba(MASKED));
    assert_eq!(inside, [20_608, 22_608, 31_832, 65_535]);
    for (frame, case, expected) in [
        (4, "before window", base),
        (5, "inclusive start", inside),
        (10, "last frame inside", inside),
        (11, "exclusive end", base),
        (13, "zero-alpha start", base),
        (15, "zero-alpha inside", base),
        (16, "after zero-alpha window", base),
    ] {
        assert_uniform_frame(&frames, frame, case, expected);
    }
    assert_opaque_alpha_plane(&frames);
}

fn media_track(id: &str, order: i32, material: &str, start: i64, duration: i64) -> Track {
    let mut clip = media_clip(&format!("itm_{id}"), material, start, duration);
    clip.visual = Some(full_visual());
    track(&format!("trk_{id}"), TrackKind::Visual, order, vec![clip])
}

fn video_material(id: &str) -> Material {
    material(
        id,
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    )
}

fn render_raw(
    canonical: ProjectEnvelope,
    assets: &BTreeMap<String, PathBuf>,
    raw: &Path,
) -> (String, Vec<u8>) {
    let delivery = prepare_delivery(canonical, assets, raw.parent().unwrap());
    let mut command = delivery
        .bundle
        .tasks()
        .iter()
        .find_map(|task| match (&task.action, task.product) {
            (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster) => Some(command.clone()),
            _ => None,
        })
        .expect("video master command");
    let graph = command.filter_graph.clone().expect("video filter graph");
    command.output_args = vec![
        "-frames:v".to_owned(),
        FRAME_COUNT.to_string(),
        "-c:v".to_owned(),
        "rawvideo".to_owned(),
        "-pix_fmt".to_owned(),
        "gbrap16le".to_owned(),
        "-f".to_owned(),
        "rawvideo".to_owned(),
        "-an".to_owned(),
    ];
    command.output_path = raw.to_owned();
    let output = Command::new("ffmpeg")
        .args(command.to_args())
        .output()
        .expect("run emitted FFmpeg command");
    assert!(
        output.status.success(),
        "FFmpeg failed: {}\n{graph}",
        String::from_utf8_lossy(&output.stderr)
    );
    let frames = fs::read(raw).expect("read raw output");
    assert_eq!(frames.len(), FRAME_COUNT * FRAME_BYTES);
    (graph, frames)
}

fn rgba_fixture(directory: &Path, name: &str, color: [u8; 4], duration: f64) -> PathBuf {
    let path = directory.join(format!("{name}.mkv"));
    let source = format!(
        "color=c=0x{:02X}{:02X}{:02X}{:02X}:s={WIDTH}x{HEIGHT}:r={FPS}:d={duration},format=gbrap16le",
        color[0], color[1], color[2], color[3]
    );
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(source)
        .args(["-c:v", "ffv1", "-level", "3", "-pix_fmt", "gbrap16le"])
        .arg(&path)
        .output()
        .expect("render lossless RGBA fixture");
    assert!(
        output.status.success(),
        "fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    path
}

fn expanded_rgba(color: [u8; 4]) -> [u16; 4] {
    color.map(|value| u16::from(value) * 257)
}

fn masked_rgb(base: [u16; 4], source: [u16; 4]) -> [u16; 4] {
    let mask = u64::from(source[3]);
    // FFmpeg maskedmerge rounds its 16-bit weighted sum to the nearest sample.
    let blend = |base: u16, source: u16| {
        let total = u64::from(base) * (65_535 - mask) + u64::from(source) * mask;
        ((total + 32_767) / 65_535) as u16
    };
    [
        blend(base[0], source[0]),
        blend(base[1], source[1]),
        blend(base[2], source[2]),
        base[3],
    ]
}

fn assert_uniform_frame(frames: &[u8], frame: usize, case: &str, expected: [u16; 4]) {
    for pixel in 0..PIXELS {
        let actual = [0, 1, 2, 3].map(|channel| sample(frames, frame, channel, pixel));
        assert_eq!(actual, expected, "{case}: frame={frame}, pixel={pixel}");
    }
}

fn assert_opaque_alpha_plane(frames: &[u8]) {
    for frame in 0..FRAME_COUNT {
        for pixel in 0..PIXELS {
            assert_eq!(sample(frames, frame, 3, pixel), u16::MAX);
        }
    }
}

fn sample(frames: &[u8], frame: usize, rgba_channel: usize, pixel: usize) -> u16 {
    let plane = [2, 0, 1, 3][rgba_channel];
    let offset = frame * FRAME_BYTES + (plane * PIXELS + pixel) * 2;
    u16::from_le_bytes([frames[offset], frames[offset + 1]])
}
