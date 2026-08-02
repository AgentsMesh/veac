use tempfile::tempdir;

use super::support::*;

#[test]
fn speed_ramp_and_hold_render_observable_frames_silence_and_exact_duration() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_retime",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Auto,
    ));
    let mut clip = media_clip("itm_retime", "med_retime", 0, 3_000);
    clip.source_mapping = Some(mapping());
    clip.audio = Some(audio_properties(1.0));
    canonical.project.sequences[0].tracks.push(track(
        "trk_retime",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    let output = temp.path().join("retimed.mp4");
    let assets = BTreeMap::from([("med_retime".to_owned(), source)]);
    let rendered = render(canonical, &assets, &output);

    assert_media_contract(&output, 1, 3.0);
    assert_color(rgb_at(&output, 0.2, 48, 26), ColorKind::Red);
    assert_color(rgb_at(&output, 0.8, 48, 26), ColorKind::Green);
    let hold_early = rgb_at(&output, 1.2, 48, 26);
    let hold_late = rgb_at(&output, 1.8, 48, 26);
    assert_color(hold_early, ColorKind::Blue);
    assert_color(hold_late, ColorKind::Blue);
    assert!(hold_early
        .iter()
        .zip(hold_late)
        .all(|(left, right)| left.abs_diff(right) <= 5));
    assert_color(rgb_at(&output, 2.2, 48, 26), ColorKind::Blue);
    assert_color(rgb_at(&output, 2.9, 48, 26), ColorKind::Yellow);

    assert!(rms_db(&audio_samples(&output, 0.2, 0.3)) > -35.0);
    assert!(rms_db(&audio_samples(&output, 1.2, 0.5)) < -55.0);
    assert!(rms_db(&audio_samples(&output, 2.2, 0.3)) > -35.0);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("rampholdv") && graph.contains("rampholda"));
}

#[test]
fn near_tail_unaligned_freeze_and_hold_select_the_displayed_frame() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_tail",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut freeze = media_clip("itm_freeze_tail", "med_tail", 0, 1_000);
    freeze.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new("med_tail").unwrap(),
        source_time: time(3_950),
    };
    freeze.source_mapping = None;
    let mut hold = media_clip("itm_hold_tail", "med_tail", 1_000, 1_000);
    hold.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![segment(1_000, 3_950, 3_950, SourceTimeInterpolation::Hold)],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    canonical.project.sequences[0].tracks.push(track(
        "trk_tail",
        TrackKind::Video,
        0,
        vec![freeze, hold],
    ));
    let output = temp.path().join("tail-holds.mp4");
    let assets = BTreeMap::from([("med_tail".to_owned(), source)]);
    let rendered = render(canonical, &assets, &output);

    assert_media_contract(&output, 0, 2.0);
    assert_color(rgb_at(&output, 0.5, 48, 26), ColorKind::Yellow);
    assert_color(rgb_at(&output, 1.5, 48, 26), ColorKind::Yellow);
    let graph = rendered.command.filter_graph.unwrap();
    assert_eq!(graph.matches("start_time=3.95:round=down").count(), 2);
}

#[test]
fn maximum_project_timebase_freeze_still_emits_only_canvas_rate_frames() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    let timescale = u32::MAX;
    let mut canonical = project(false);
    canonical.project.timebase = timescale;
    canonical.project.materials.push(material(
        "med_precise",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut freeze = media_clip("itm_precise", "med_precise", 0, 1_000);
    freeze.record_range = TimeRange::new(
        RationalTime::new(0, timescale).unwrap(),
        RationalTime::new(i64::from(timescale), timescale).unwrap(),
    )
    .unwrap();
    freeze.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new("med_precise").unwrap(),
        source_time: RationalTime::new(i64::from(timescale) * 395 / 100, timescale).unwrap(),
    };
    freeze.source_mapping = None;
    canonical.project.sequences[0].tracks.push(track(
        "trk_precise",
        TrackKind::Video,
        0,
        vec![freeze],
    ));
    let output = temp.path().join("precise-freeze.mp4");
    let assets = BTreeMap::from([("med_precise".to_owned(), source)]);
    let rendered = render(canonical, &assets, &output);

    assert_media_contract(&output, 0, 1.0);
    assert_eq!(video_frame_count(&output), FPS as u64);
    assert_color(rgb_at(&output, 0.5, 48, 26), ColorKind::Yellow);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("fps=fps=10/1:start_time="));
    assert!(!graph.contains("fps=fps=4294967295"));
}

fn mapping() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(1_000, 0, 2_000, SourceTimeInterpolation::Linear),
                segment(1_000, 2_000, 2_000, SourceTimeInterpolation::Hold),
                segment(1_000, 2_000, 3_200, SourceTimeInterpolation::Linear),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    interpolation: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation,
    }
}

enum ColorKind {
    Red,
    Green,
    Blue,
    Yellow,
}

fn assert_color(pixel: [u8; 3], expected: ColorKind) {
    let valid = match expected {
        ColorKind::Red => pixel[0] > 180 && pixel[1] < 80 && pixel[2] < 80,
        ColorKind::Green => pixel[1] > 70 && pixel[0] < 80 && pixel[2] < 80,
        ColorKind::Blue => pixel[2] > 180 && pixel[0] < 80 && pixel[1] < 80,
        ColorKind::Yellow => pixel[0] > 180 && pixel[1] > 180 && pixel[2] < 80,
    };
    assert!(valid, "unexpected pixel {pixel:?}");
}
