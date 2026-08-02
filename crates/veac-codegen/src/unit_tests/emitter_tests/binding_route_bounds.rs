use veac_artifact::{ArtifactKind, ArtifactStore, MediaRole, SourceClock};
use veac_codegen::emitter::emit_all;
use veac_plan::canonical::{PlaybackDirection, SourceTimeInterpolation, SourceTimeSegment};
use veac_plan::{ResolvedClipSource, ResolvedSourceTimeMap};

use super::binding_routes::{proxy, range, typed, video_command};
use super::support::{fixture, resolved, time};

#[test]
fn nonzero_clock_maps_linear_reverse_source_time_to_proxy_time() {
    let mut plan = resolved(&fixture());
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    let mapping = clip.source_mapping.as_mut().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        direction,
        ..
    } = &mut mapping.time_map
    else {
        panic!("fixture mapping must be linear")
    };
    source_range_per_repeat.start = time(1_200);
    *direction = PlaybackDirection::Reverse;
    let temp = tempfile::tempdir().unwrap();
    let local = proxy_bindings(
        &plan,
        &ArtifactStore::new(temp.path()),
        SourceClock::bounded(range(1_200, 600), time(0)).unwrap(),
    );
    let bundle = emit_all(&plan, &local).unwrap();
    let graph = video_command(&bundle).filter_graph.as_deref().unwrap();
    assert!(graph.contains("trim=start=0:duration=1"), "{graph}");
    assert!(graph.contains("reverse"), "{graph}");
}

#[test]
fn nonzero_clock_maps_curve_ramps_before_speed_processing() {
    let mut plan = resolved(&fixture());
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    let mapping = clip.source_mapping.as_mut().unwrap();
    mapping.time_map = ResolvedSourceTimeMap::Curve {
        segments: vec![SourceTimeSegment {
            record_duration: time(600),
            source_start: time(1_200),
            source_end: time(1_800),
            interpolation: SourceTimeInterpolation::Linear,
        }],
    };
    let temp = tempfile::tempdir().unwrap();
    let local = proxy_bindings(
        &plan,
        &ArtifactStore::new(temp.path()),
        SourceClock::bounded(range(1_200, 600), time(0)).unwrap(),
    );
    let bundle = emit_all(&plan, &local).unwrap();
    let graph = video_command(&bundle).filter_graph.as_deref().unwrap();
    assert!(graph.contains("trim=start=0:duration=1"), "{graph}");
    assert!(graph.contains("setpts=PTS*600/600"), "{graph}");
}

#[test]
fn freeze_uses_mapped_proxy_time_and_rejects_the_half_open_end() {
    let mut plan = resolved(&fixture());
    let input = plan.inputs[0].clone();
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    clip.source = ResolvedClipSource::FreezeFrame {
        input_id: input.id.clone(),
        video_stream: input.video.as_ref().unwrap().selection,
        source_time: time(1_500),
    };
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let local = proxy_bindings(
        &plan,
        &store,
        SourceClock::bounded(range(1_200, 600), time(0)).unwrap(),
    );
    let bundle = emit_all(&plan, &local).unwrap();
    let graph = video_command(&bundle).filter_graph.as_deref().unwrap();
    assert!(graph.contains("start_time=0.5"), "{graph}");

    let ResolvedClipSource::FreezeFrame { source_time, .. } =
        &mut plan.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    *source_time = time(1_800);
    let error = emit_all(&plan, &local).unwrap_err();
    assert!(error.to_string().contains("outside bound media"));
}

#[test]
fn reverse_and_ramp_ranges_must_be_fully_covered_by_the_proxy() {
    let mut plan = resolved(&fixture());
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    let mapping = clip.source_mapping.as_mut().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        direction,
        ..
    } = &mut mapping.time_map
    else {
        unreachable!()
    };
    source_range_per_repeat.start = time(1_200);
    *direction = PlaybackDirection::Reverse;
    let temp = tempfile::tempdir().unwrap();
    let local = proxy_bindings(
        &plan,
        &ArtifactStore::new(temp.path()),
        SourceClock::bounded(range(1_200, 597), time(0)).unwrap(),
    );
    let error = emit_all(&plan, &local).unwrap_err();
    assert!(error
        .to_string()
        .contains("source range exceeds its binding"));
}

fn proxy_bindings(
    plan: &veac_plan::ResolvedRenderPlan,
    store: &ArtifactStore,
    clock: SourceClock,
) -> veac_artifact::ExecutionBindings {
    let input = &plan.inputs[0];
    let artifact = proxy(
        store,
        input,
        ArtifactKind::ProxyVideo,
        b"video proxy",
        clock,
    );
    let mut local = typed(plan);
    local
        .bind_verified_proxy(input, MediaRole::Video, &artifact)
        .unwrap();
    local
}
