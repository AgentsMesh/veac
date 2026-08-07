use super::support::*;

#[test]
fn fractional_frame_transition_completes_without_a_frozen_tail() {
    for fps in [12, 24] {
        assert_transition_frames(fps);
    }
}

#[test]
fn fractional_custom_wipe_reaches_midpoint_on_exact_cut_at_twelve_fps() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("exact-cut-wipe.mp4");
    render(
        gallery_wipe_project(),
        &std::collections::BTreeMap::new(),
        &output,
    );

    let left = rgb_at(&output, 5.0, WIDTH / 4, HEIGHT / 2);
    let right = rgb_at(&output, 5.0, WIDTH * 3 / 4, HEIGHT / 2);
    assert!(
        i16::from(left[0]) >= i16::from(right[0]) + 45,
        "custom wipe was not at its midpoint on the cut frame: left={left:?}, right={right:?}"
    );
}

#[test]
fn subframe_transition_uses_real_endpoint_windows_without_a_frozen_tail() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("subframe-transition.mp4");
    let rendered = render(
        transition_project(12, 40, TransitionKind::Dissolve),
        &std::collections::BTreeMap::new(),
        &output,
    );
    let graph = rendered.command.filter_graph.unwrap();

    assert!(
        graph.contains("trim=start=1.96:duration=0.04,setpts=PTS-STARTPTS")
            && graph.contains("trim=start=0:duration=0.04,setpts=PTS-STARTPTS")
            && graph.contains("xfade=transition=fade:duration=0.083333333333:offset=0")
            && !graph.contains("reverse"),
        "graph={graph}"
    );
    let outgoing = rgb_at(&output, 1.9, WIDTH / 2, HEIGHT / 2);
    assert!(outgoing[0] > 220 && outgoing[2] < 20, "{outgoing:?}");
    let completed = rgb_at(&output, 2.0, WIDTH / 2, HEIGHT / 2);
    assert!(
        completed[2] > 220 && completed[0] < 20,
        "completed={completed:?}"
    );
}

fn assert_transition_frames(fps: i64) {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join(format!("fractional-transition-{fps}.mp4"));
    render(
        transition_project(fps, 400, TransitionKind::Dissolve),
        &std::collections::BTreeMap::new(),
        &output,
    );
    let first = centered_frame_at_or_after(1.6, fps);
    let after = centered_frame_at_or_after(2.0, fps);
    let progress: Vec<_> = (first..after)
        .map(|frame| {
            blue_progress(rgb_at(
                &output,
                (frame as f64 + 0.5) / fps as f64,
                WIDTH / 2,
                HEIGHT / 2,
            ))
        })
        .collect();

    assert!(progress.len() >= 4, "fps={fps} progress={progress:?}");
    assert!(
        progress.windows(2).all(|pair| pair[1] + 0.03 >= pair[0]),
        "fps={fps} progress={progress:?}"
    );
    assert!(
        progress.first().copied().unwrap() < 0.35 && progress.last().copied().unwrap() > 0.65,
        "fps={fps} progress={progress:?}"
    );
    let advancing = progress
        .windows(2)
        .filter(|pair| pair[1] > pair[0] + 0.04)
        .count();
    assert!(advancing >= 3, "fps={fps} progress={progress:?}");
    for frame in [after, after + 1] {
        let rgb = rgb_at(
            &output,
            (frame as f64 + 0.5) / fps as f64,
            WIDTH / 2,
            HEIGHT / 2,
        );
        assert!(
            rgb[2] > 220 && rgb[0] < 20,
            "fps={fps} frame={frame} rgb={rgb:?}"
        );
    }
}

fn transition_project(fps: i64, duration_ms: i64, kind: TransitionKind) -> ProjectEnvelope {
    let mut canonical = project(false);
    canonical.project.sequences[0].settings.frame_rate = ratio(fps, 1);
    let raster = canonical.project.render_configs[0].raster.as_mut().unwrap();
    raster.frame_rate = ratio(fps, 1);
    canonical.project.sequences[0].tracks.push(track(
        "trk_video",
        TrackKind::Video,
        0,
        vec![
            visual_solid_clip("itm_red", color(255, 0, 0), 0, 2_000),
            visual_solid_clip("itm_blue", color(0, 0, 255), 2_000 - duration_ms, 2_000),
        ],
    ));
    canonical.project.relations.push(Relation {
        id: RelationId::new("rel_fractional_frames").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Transition {
            from: RelationEndpoint::item(ItemId::new("itm_red").unwrap()),
            to: RelationEndpoint::item(ItemId::new("itm_blue").unwrap()),
            transition: Transition {
                kind,
                duration: time(duration_ms),
                alignment: TransitionAlignment::Centered,
            },
        },
    });
    canonical
}

fn left_wipe() -> TransitionKind {
    TransitionKind::Wipe {
        direction: CardinalDirection::Left,
        angle_degrees: 0.0,
        softness: 0.05,
    }
}

fn gallery_wipe_project() -> ProjectEnvelope {
    let mut canonical = project(false);
    canonical.project.sequences[0].settings.frame_rate = ratio(12, 1);
    let raster = canonical.project.render_configs[0].raster.as_mut().unwrap();
    raster.frame_rate = ratio(12, 1);
    canonical.project.sequences[0].tracks.push(track(
        "trk_gallery",
        TrackKind::Video,
        0,
        vec![
            visual_solid_clip("itm_a0", color(255, 190, 11), 0, 1_175),
            visual_solid_clip("itm_b0", color(251, 86, 7), 825, 1_175),
            visual_solid_clip("itm_a1", color(255, 0, 110), 2_000, 1_175),
            visual_solid_clip("itm_b1", color(131, 56, 236), 2_825, 1_175),
            visual_solid_clip("itm_wipe_a", color(58, 134, 255), 4_000, 1_175),
            visual_solid_clip("itm_wipe_b", color(0, 180, 216), 4_825, 1_175),
        ],
    ));
    for (from, to, kind) in [
        ("itm_a0", "itm_b0", TransitionKind::Dissolve),
        (
            "itm_a1",
            "itm_b1",
            TransitionKind::Fade {
                color: FadeColor::Black,
            },
        ),
        ("itm_wipe_a", "itm_wipe_b", left_wipe()),
    ] {
        add_transition(
            &mut canonical,
            "seq_main",
            from,
            to,
            Transition {
                kind,
                duration: time(350),
                alignment: TransitionAlignment::Centered,
            },
        );
    }
    canonical
}

fn centered_frame_at_or_after(seconds: f64, fps: i64) -> i64 {
    (seconds * fps as f64 - 0.5).ceil() as i64
}

fn blue_progress(rgb: [u8; 3]) -> f64 {
    f64::from(rgb[2]) / (f64::from(rgb[0]) + f64::from(rgb[2])).max(1.0)
}
