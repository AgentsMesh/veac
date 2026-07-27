use std::process::Command;

use tempfile::tempdir;

use super::support::*;

#[test]
fn horizontal_vertical_and_combined_flips_move_real_quadrants() {
    let temp = tempdir().unwrap();
    let source = quadrant_fixture(temp.path());
    let output = temp.path().join("flips.mp4");
    let mut value = project(false);
    value.project.materials.push(material(
        "med_quadrants",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let clips = [
        flipped_clip("itm_horizontal", 0, true, false),
        flipped_clip("itm_vertical", 1_000, false, true),
        flipped_clip("itm_both", 2_000, true, true),
    ];
    value.project.sequences[0]
        .tracks
        .push(track("trk_flips", TrackKind::Video, 0, clips.into()));
    render(
        value,
        &BTreeMap::from([("med_quadrants".to_owned(), source)]),
        &output,
    );

    assert_quadrants(
        &output,
        0.5,
        [Tone::Green, Tone::Red, Tone::Yellow, Tone::Blue],
    );
    assert_quadrants(
        &output,
        1.5,
        [Tone::Blue, Tone::Yellow, Tone::Red, Tone::Green],
    );
    assert_quadrants(
        &output,
        2.5,
        [Tone::Yellow, Tone::Blue, Tone::Green, Tone::Red],
    );
}

fn flipped_clip(id: &str, start: i64, horizontal: bool, vertical: bool) -> Clip {
    let mut clip = media_clip(id, "med_quadrants", start, 1_000);
    let mut visual = full_visual();
    visual.transform.flip_horizontal = horizontal;
    visual.transform.flip_vertical = vertical;
    clip.visual = Some(visual);
    clip
}

fn quadrant_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("quadrants.mp4");
    let filter = format!(
        "color=c=red:s={WIDTH}x{HEIGHT}:r={FPS}:d=3,drawbox=x={}:y=0:w={}:h={}:color=lime:t=fill,drawbox=x=0:y={}:w={}:h={}:color=blue:t=fill,drawbox=x={}:y={}:w={}:h={}:color=yellow:t=fill",
        WIDTH / 2,
        WIDTH / 2,
        HEIGHT / 2,
        HEIGHT / 2,
        WIDTH / 2,
        HEIGHT / 2,
        WIDTH / 2,
        HEIGHT / 2,
        WIDTH / 2,
        HEIGHT / 2,
    );
    let result = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(filter)
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
        .arg(&output)
        .output()
        .expect("start quadrant fixture");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}

fn assert_quadrants(path: &Path, second: f64, expected: [Tone; 4]) {
    let points = [(12, 10), (84, 10), (12, 44), (84, 44)];
    for (point, expected) in points.into_iter().zip(expected) {
        expected.assert(rgb_at(path, second, point.0, point.1));
    }
}

#[derive(Clone, Copy)]
enum Tone {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Tone {
    fn assert(self, pixel: [u8; 3]) {
        let matches = match self {
            Self::Red => pixel[0] > 170 && pixel[1] < 80 && pixel[2] < 80,
            Self::Green => pixel[1] > 140 && pixel[0] < 80 && pixel[2] < 80,
            Self::Blue => pixel[2] > 170 && pixel[0] < 80 && pixel[1] < 100,
            Self::Yellow => pixel[0] > 160 && pixel[1] > 160 && pixel[2] < 100,
        };
        assert!(matches, "unexpected pixel {pixel:?}");
    }
}
