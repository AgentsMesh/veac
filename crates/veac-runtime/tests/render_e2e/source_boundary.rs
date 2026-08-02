use std::process::Command;

use tempfile::tempdir;

use super::support::*;
use cases::BoundaryCase;

#[path = "source_boundary/cases.rs"]
mod cases;

#[test]
fn original_av_boundaries_clone_video_and_pad_audio_with_silence() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    for case in [BoundaryCase::first(), BoundaryCase::last()] {
        let mut canonical = project(true);
        canonical.project.materials.push(material(
            "med_boundary",
            MaterialKind::Video,
            StreamChoice::Auto,
            StreamChoice::Auto,
        ));
        let mut clip = media_clip("itm_boundary", "med_boundary", 0, 1_500);
        clip.source_mapping = Some(boundary_mapping(case.start, case.policy));
        clip.audio = Some(audio_properties(1.0));
        canonical.project.sequences[0].tracks.push(track(
            "trk_boundary",
            TrackKind::Video,
            0,
            vec![clip],
        ));
        let output = temp.path().join(format!("{}.mp4", case.name));
        let rendered = render(
            canonical,
            &BTreeMap::from([("med_boundary".to_owned(), source.clone())]),
            &output,
        );

        assert_media_contract(&output, 1, 1.5);
        assert_color(rgb_at(&output, case.pixel_time, 48, 26), case.color);
        assert!(rms_db(&audio_samples(&output, case.silent_at, 0.25)) < -55.0);
        assert!(rms_db(&audio_samples(&output, case.signal_at, 0.25)) > -35.0);
        let graph = rendered.command.filter_graph.unwrap();
        assert!(graph.contains(case.video_filter), "{graph}");
        assert!(graph.contains(case.audio_filter), "{graph}");
    }
}

#[test]
fn nested_sequence_can_hold_both_source_boundaries() {
    let temp = tempdir().unwrap();
    let child = sequence(
        "seq_child_boundary",
        vec![track(
            "trk_child_boundary",
            TrackKind::Video,
            0,
            vec![
                solid_clip("itm_child_red", color(255, 0, 0), 0, 1_000),
                solid_clip("itm_child_blue", color(0, 0, 255), 1_000, 1_000),
            ],
        )],
    );
    let mut parent = nested_clip("itm_parent_boundary", "seq_child_boundary", 0, 4_000);
    parent.source_mapping = Some(boundary_mapping(-1_000, SourceOutOfRangePolicy::HoldBoth));
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_parent_boundary",
        TrackKind::Video,
        0,
        vec![parent],
    ));
    canonical.project.sequences.push(child);
    let output = temp.path().join("nested-boundary.mp4");
    let rendered = render(canonical, &BTreeMap::new(), &output);

    assert_media_contract(&output, 0, 4.0);
    assert_color(rgb_at(&output, 0.5, 48, 26), ExpectedColor::Red);
    assert_color(rgb_at(&output, 3.5, 48, 26), ExpectedColor::Blue);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("start_mode=clone"), "{graph}");
    assert!(graph.contains("stop_mode=clone"), "{graph}");
}

#[test]
fn still_image_ignores_a_large_source_coordinate() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("still.png");
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=96x54",
            "-frames:v",
            "1",
            "-update",
            "1",
        ])
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_still",
        MaterialKind::Image,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut clip = media_clip("itm_still", "med_still", 0, 1_000);
    clip.source_mapping = Some(boundary_mapping(60_000, SourceOutOfRangePolicy::Strict));
    canonical.project.sequences[0]
        .tracks
        .push(track("trk_still", TrackKind::Video, 0, vec![clip]));
    let rendered_path = temp.path().join("still-large-coordinate.mp4");
    let rendered = render(
        canonical,
        &BTreeMap::from([("med_still".to_owned(), source)]),
        &rendered_path,
    );

    assert_media_contract(&rendered_path, 0, 1.0);
    assert_color(rgb_at(&rendered_path, 0.5, 48, 26), ExpectedColor::Red);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(
        graph.contains("loop=loop=-1:size=1:start=0,trim=start=0"),
        "{graph}"
    );
    assert!(!graph.contains("trim=start=60"), "{graph}");
}

fn boundary_mapping(start: i64, policy: SourceOutOfRangePolicy) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(start),
            rate: ratio(1, 1),
            repeat: 1,
            direction: PlaybackDirection::Forward,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: policy,
    }
}

#[derive(Clone, Copy)]
enum ExpectedColor {
    Red,
    Yellow,
    Blue,
}

fn assert_color(pixel: [u8; 3], color: ExpectedColor) {
    let valid = match color {
        ExpectedColor::Red => pixel[0] > 180 && pixel[1] < 80 && pixel[2] < 80,
        ExpectedColor::Yellow => pixel[0] > 180 && pixel[1] > 180 && pixel[2] < 80,
        ExpectedColor::Blue => pixel[2] > 180 && pixel[0] < 80 && pixel[1] < 80,
    };
    assert!(valid, "unexpected pixel {pixel:?}");
}
