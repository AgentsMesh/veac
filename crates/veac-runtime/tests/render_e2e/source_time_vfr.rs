use tempfile::tempdir;
use veac_artifact::ExecutionBindings;
use veac_codegen::emitter;
use veac_plan::resolve_one;

use super::support::*;

#[path = "source_time_vfr/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn real_asymmetric_vfr_pts_are_probed_and_reverse_fails_before_ffmpeg() {
    let temp = tempdir().unwrap();
    let source = asymmetric_vfr_fixture(temp.path());
    assert_eq!(frame_pts(&source), vec![0.0, 0.1, 0.4, 0.9]);

    assert_vfr_reverse_rejected(source, ratio(4, 1), 1_000);
}

#[test]
fn equal_stream_summary_rates_cannot_hide_real_vfr_packet_cadence() {
    let temp = tempdir().unwrap();
    let source = equal_rate_vfr_fixture(temp.path());
    assert_eq!(
        frame_pts(&source),
        vec![0.0, 0.05, 0.2, 0.25, 0.4, 0.45, 0.6, 0.65]
    );
    assert_eq!(stream_rates(&source), "20/1,20/1");

    assert_vfr_reverse_rejected(source, ratio(20, 1), 600);
}

#[test]
fn real_fractional_cfr_with_coarse_timebase_can_reverse() {
    let temp = tempdir().unwrap();
    let source = fractional_cfr_fixture(temp.path());
    assert_eq!(packet_ticks(&source)[..6], [0, 33, 67, 100, 133, 167]);

    let project = reverse_project(&source, 400);
    let info = project.project.materials[0].probe.as_ref().unwrap().streams[0]
        .video
        .as_ref()
        .unwrap();
    assert_eq!(info.frame_rate, Some(ratio(30_000, 1_001)));
    assert_eq!(info.cadence, VideoCadence::Constant);

    let plan = resolve_project(&project);
    let mut bindings = source_bindings(&plan, source);
    bindings
        .bind_output(
            plan.output.deliverables[0].id.clone(),
            temp.path().join("out.mp4"),
        )
        .unwrap();
    emitter::emit_all(&plan, &bindings).unwrap();
}

fn assert_vfr_reverse_rejected(source: PathBuf, expected_rate: Rational, duration_ms: i64) {
    let project = reverse_project(&source, duration_ms);
    let info = project.project.materials[0].probe.as_ref().unwrap().streams[0]
        .video
        .as_ref()
        .unwrap();
    assert_eq!(info.frame_rate, Some(expected_rate));
    assert_eq!(info.cadence, VideoCadence::Variable);

    let plan = resolve_project(&project);
    let mut bindings = source_bindings(&plan, source);
    bindings
        .bind_output(
            plan.output.deliverables[0].id.clone(),
            "/tmp/vfr-out.mp4".into(),
        )
        .unwrap();
    let error = emitter::emit_all(&plan, &bindings).unwrap_err();
    let cadence = error
        .diagnostics()
        .iter()
        .find(|value| value.code == "PLAN_REVERSE_CADENCE_UNSUPPORTED")
        .expect("VFR reverse diagnostic");
    assert!(cadence.message.contains("transcode the source to CFR"));
}

fn reverse_project(source: &Path, duration_ms: i64) -> ProjectEnvelope {
    let mut project = project(false);
    project.project.materials.push(material(
        "med_vfr_reverse",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut clip = media_clip("itm_vfr_reverse", "med_vfr_reverse", 0, duration_ms);
    let SourceTimeMap::Linear { direction, .. } =
        &mut clip.source_mapping.as_mut().unwrap().time_map
    else {
        unreachable!()
    };
    *direction = PlaybackDirection::Reverse;
    project.project.sequences[0].tracks =
        vec![track("trk_vfr_reverse", TrackKind::Video, 0, vec![clip])];
    hydrate(
        &mut project,
        &BTreeMap::from([("med_vfr_reverse".to_owned(), source.to_owned())]),
    );
    project
}

fn resolve_project(project: &ProjectEnvelope) -> veac_plan::ResolvedRenderPlan {
    let output_id = project.project.render_configs[0].id.clone();
    resolve_one(project, &output_id).unwrap()
}

fn source_bindings(plan: &veac_plan::ResolvedRenderPlan, source: PathBuf) -> ExecutionBindings {
    ExecutionBindings::from_originals(plan, &BTreeMap::from([(plan.inputs[0].id.clone(), source)]))
        .unwrap()
}
