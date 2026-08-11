use veac_plan::canonical::{ItemId, SequenceId, TrackId};
use veac_plan::ResolvedClipSource;

use super::support::*;

#[test]
fn child_graph_and_parent_reverse_accumulate_in_one_delivery() {
    let mut plan = nested_plan();
    reverse_clip(&mut plan.sequences.last_mut().unwrap().tracks[0].clips[0]);

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

#[test]
fn repeated_nested_consumers_accumulate_even_without_record_overlap() {
    let mut plan = nested_plan();
    let parent = plan.sequences.last_mut().unwrap();
    let mut second = parent.tracks[0].clone();
    second.id = TrackId::new("trk_nested_second").unwrap();
    second.order = 1;
    second.source_order = 1;
    second.clips[0].id = ItemId::new("itm_nested_second").unwrap();
    second.clips[0].record_range.start = time(600);
    parent.tracks.push(second);
    parent.duration = time(1_200);

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

fn nested_plan() -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&fixture());
    let mut child = plan.sequences[0].clone();
    child.id = SequenceId::new("seq_reverse_child").unwrap();
    child.tracks[0].id = TrackId::new("trk_reverse_child").unwrap();
    child.tracks[0].clips[0].id = ItemId::new("itm_reverse_child").unwrap();
    reverse_clip(&mut child.tracks[0].clips[0]);

    plan.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    plan.sequences.insert(0, child);
    plan
}
