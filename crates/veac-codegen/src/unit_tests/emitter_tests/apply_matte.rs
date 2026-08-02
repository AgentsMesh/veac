use veac_plan::canonical::{RelationId, TrackMatteMode};
use veac_plan::ResolvedMatte;

use super::composition_advanced::advanced_plan;
use super::support::{bindings, emit_video_command};

#[test]
fn apply_matte_runs_after_stages_and_before_result_mix() {
    let mut plan = advanced_plan();
    let source = plan.sequences[0].tracks[1].clips[0].id.clone();
    plan.sequences[0].applies[0].matte = Some(ResolvedMatte {
        relation_id: RelationId::new("rel_apply_matte_runtime").unwrap(),
        source_clip_id: source,
        mode: TrackMatteMode::Alpha,
        invert: false,
    });
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let stage = graph.find("applystagev").unwrap();
    let matte = stage + graph[stage..].find("alphaextract,format=gray16le").unwrap();
    let mix = graph.find("applymixsplitv").unwrap();
    assert!(stage < matte, "stage must precede matte: {graph}");
    assert!(
        mix < stage,
        "original/process split must precede stages: {graph}"
    );
    assert!(graph.contains("mergeplanes=format=gbrap16le"));
}
