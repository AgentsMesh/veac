use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn partial_stabilization_splices_only_the_active_interval() {
    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].effects = vec![ResolvedEffect {
        id: EffectId::new("fx_partial_stabilize").unwrap(),
        effect_type: "video.stabilize".into(),
        active_range: TimeRange::new(time(100), time(300)).unwrap(),
        parameters: [("enabled".into(), ParameterValue::Boolean { value: true })]
            .into_iter()
            .collect(),
    }];
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    for marker in [
        "split=3",
        "trim=start=0:end=0.166666666667",
        "trim=start=0.166666666667:end=0.666666666667",
        "deshake",
        "trim=start=0.666666666667:end=1",
        "concat=n=3:v=1:a=0",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert!(!graph.contains("deshake=enable="), "{graph}");
}
