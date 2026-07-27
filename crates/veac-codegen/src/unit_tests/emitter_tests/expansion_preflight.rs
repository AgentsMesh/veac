use veac_codegen::emitter::emit_all;
use veac_plan::canonical::{ItemId, SequenceId, TrackId};
use veac_plan::ResolvedClipSource;

use super::support::{bindings, fixture, resolved};

#[test]
fn nested_sequence_expansion_is_bounded_before_graph_construction() {
    let mut plan = resolved(&fixture());
    let mut child = plan.sequences.remove(0);
    child.id = SequenceId::new("seq_expand_0").unwrap();
    let mut sequences = vec![child.clone()];
    for depth in 1..=15 {
        let mut parent = child.clone();
        parent.id = SequenceId::new(format!("seq_expand_{depth}")).unwrap();
        parent.tracks[0].id = TrackId::new(format!("trk_expand_{depth}")).unwrap();
        let mut first = parent.tracks[0].clips[0].clone();
        first.id = ItemId::new(format!("itm_expand_{depth}_a")).unwrap();
        first.source_order = 0;
        first.source = ResolvedClipSource::Sequence {
            sequence_id: child.id.clone(),
        };
        let mut second = first.clone();
        second.id = ItemId::new(format!("itm_expand_{depth}_b")).unwrap();
        second.source_order = 1;
        parent.tracks[0].clips = vec![first, second];
        child = parent.clone();
        sequences.push(parent);
    }
    plan.entry_sequence_id = child.id.clone();
    plan.output.sequence_id = child.id.clone();
    plan.sequences = sequences;

    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();

    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_EXPANSION_LIMIT"));
}

#[test]
fn nested_sequence_depth_is_rejected_before_recursive_emission() {
    let mut plan = resolved(&fixture());
    let mut child = plan.sequences.remove(0);
    child.id = SequenceId::new("seq_depth_0").unwrap();
    let mut sequences = vec![child.clone()];
    for depth in 1..=veac_plan::canonical::MAX_SEQUENCE_NESTING_DEPTH {
        let mut parent = child.clone();
        parent.id = SequenceId::new(format!("seq_depth_{depth}")).unwrap();
        parent.tracks[0].id = TrackId::new(format!("trk_depth_{depth}")).unwrap();
        parent.tracks[0].clips[0].id = ItemId::new(format!("itm_depth_{depth}")).unwrap();
        parent.tracks[0].clips[0].effects.clear();
        parent.tracks[0].clips[0].source = ResolvedClipSource::Sequence {
            sequence_id: child.id.clone(),
        };
        child = parent.clone();
        sequences.push(parent);
    }
    plan.entry_sequence_id = child.id.clone();
    plan.output.sequence_id = child.id.clone();
    plan.sequences = sequences;

    let result = std::panic::catch_unwind(|| emit_all(&plan, &bindings(&plan)));
    let error = result.unwrap().unwrap_err();

    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_SEQUENCE_DEPTH_LIMIT"));
}
