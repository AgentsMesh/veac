use std::process::Command;
use tempfile::tempdir;

use super::support::*;

#[test]
fn nested_media_reverse_curve_preserves_motion_across_probe_timebase() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_nested_curve",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut media = media_clip("itm_nested_media", "med_nested_curve", 0, 3_000);
    media.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![SourceTimeSegment {
                record_duration: time(3_000),
                source_start: time(3_900),
                source_end: time(0),
                interpolation: SourceTimeInterpolation::Linear,
            }],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    let child = sequence(
        "seq_nested_curve",
        vec![track("trk_nested_media", TrackKind::Visual, 0, vec![media])],
    );
    canonical.project.sequences[0].tracks.push(track(
        "trk_nested_curve",
        TrackKind::Visual,
        0,
        vec![nested_clip(
            "itm_nested_curve",
            "seq_nested_curve",
            0,
            3_000,
        )],
    ));
    canonical.project.sequences.push(child);
    let output = temp.path().join("nested-reverse-curve.mp4");
    let assets = BTreeMap::from([("med_nested_curve".to_owned(), source)]);

    let rendered = render(canonical, &assets, &output);

    assert_media_contract(&output, 0, 3.0);
    assert_color(rgb_at(&output, 0.2, 48, 26), [true, true, false]);
    assert_color(rgb_at(&output, 1.0, 48, 26), [false, false, true]);
    assert_color(rgb_at(&output, 2.0, 48, 26), [false, true, false]);
    assert_color(rgb_at(&output, 2.8, 48, 26), [true, false, false]);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("rampreversev"), "{graph}");
    assert!(!graph.contains("frameboundv"), "{graph}");
    assert!(!graph.contains("setpts=N*"), "{graph}");
}

#[test]
fn nested_short_reverse_segments_cover_a_24_fps_source_on_a_10_fps_canvas() {
    let temp = tempdir().unwrap();
    let source = time_ramp_fixture(temp.path());
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_nested_ramp",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut media = media_clip("itm_nested_ramp_source", "med_nested_ramp", 0, 1_600);
    media.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: [(2_000, 1_500), (1_500, 1_000), (1_000, 500), (500, 0)]
                .map(|(start, end)| SourceTimeSegment {
                    record_duration: time(400),
                    source_start: time(start),
                    source_end: time(end),
                    interpolation: SourceTimeInterpolation::Linear,
                })
                .to_vec(),
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    let child = sequence(
        "seq_nested_ramp",
        vec![track(
            "trk_nested_ramp_source",
            TrackKind::Visual,
            0,
            vec![media],
        )],
    );
    canonical.project.sequences[0].tracks.push(track(
        "trk_nested_ramp_wrapper",
        TrackKind::Visual,
        0,
        vec![nested_clip(
            "itm_nested_ramp_wrapper",
            "seq_nested_ramp",
            0,
            1_600,
        )],
    ));
    canonical.project.sequences.push(child);
    let output = temp.path().join("nested-short-reverse-segments.mp4");
    let assets = BTreeMap::from([("med_nested_ramp".to_owned(), source)]);

    let rendered = render(canonical, &assets, &output);

    assert_media_contract(&output, 0, 1.6);
    assert_eq!(video_frame_count(&output), 16);
    let expectations = [
        (0.1, 239),
        (0.3, 207),
        (0.5, 175),
        (0.7, 143),
        (0.9, 112),
        (1.1, 80),
        (1.3, 48),
        (1.5, 16),
    ];
    let samples = expectations.map(|(at, _)| rgb_at(&output, at, 48, 26));
    for ((at, expected), pixel) in expectations.into_iter().zip(samples) {
        assert!(pixel[0].abs_diff(expected) <= 24, "at={at}: {pixel:?}");
        assert!(pixel[1] < 30 && pixel[2] < 30, "at={at}: {pixel:?}");
    }
    let graph = rendered.command.filter_graph.unwrap();
    assert_eq!(graph.matches("]reverse[rampr").count(), 4, "{graph}");
    assert!(!graph.contains("frameboundv"), "{graph}");
    assert!(!graph.contains("reversefpsv"), "{graph}");
    assert!(!graph.contains("setpts=N*"), "{graph}");
}

fn assert_color(pixel: [u8; 3], dominant: [bool; 3]) {
    for (channel, should_dominate) in pixel.into_iter().zip(dominant) {
        if should_dominate {
            assert!(channel > 70, "pixel={pixel:?}");
        } else {
            assert!(channel < 80, "pixel={pixel:?}");
        }
    }
}

fn time_ramp_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("time-ramp-24fps.mkv");
    let result = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "nullsrc=s=96x54:r=24:d=2.1",
            "-vf",
            "geq=r='clip(N*255/47,0,255)':g=0:b=0",
            "-c:v",
            "ffv1",
            "-pix_fmt",
            "gbrp",
        ])
        .arg(&output)
        .output()
        .expect("start 24 fps fixture render");
    assert!(
        result.status.success(),
        "24 fps fixture failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}
