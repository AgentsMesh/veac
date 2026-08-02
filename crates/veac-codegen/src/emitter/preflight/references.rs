use std::collections::{BTreeMap, BTreeSet};

use veac_plan::{ResolvedClipSource, ResolvedRenderPlan, ResolvedTrack};

use super::Check;

mod complexity;

pub(super) fn references(check: &mut Check, plan: &ResolvedRenderPlan) {
    let inputs: BTreeSet<_> = plan.inputs.iter().map(|input| &input.id).collect();
    let sequences: BTreeSet<_> = plan.sequences.iter().map(|sequence| &sequence.id).collect();
    if !sequences.contains(&plan.entry_sequence_id) {
        check.push(
            "PLAN_ENTRY_MISSING",
            Some(plan.entry_sequence_id.to_string()),
            "entry sequence is absent",
        );
    }
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        match &clip.source {
            ResolvedClipSource::Media { input_id, .. }
            | ResolvedClipSource::FreezeFrame { input_id, .. }
                if !inputs.contains(input_id) =>
            {
                check.push(
                    "PLAN_INPUT_MISSING",
                    Some(clip.id.to_string()),
                    "clip input is absent from the plan",
                )
            }
            ResolvedClipSource::Sequence { sequence_id } if !sequences.contains(sequence_id) => {
                check.push(
                    "PLAN_SEQUENCE_MISSING",
                    Some(clip.id.to_string()),
                    "nested sequence is absent from the plan",
                );
            }
            ResolvedClipSource::Multicam { source }
                if source
                    .angles
                    .iter()
                    .any(|angle| !inputs.contains(&angle.input_id)) =>
            {
                check.push(
                    "PLAN_INPUT_MISSING",
                    Some(clip.id.to_string()),
                    "multicam angle input is absent from the plan",
                );
            }
            _ => {}
        }
    }
    for track in plan.sequences.iter().flat_map(|sequence| &sequence.tracks) {
        transition_endpoints(check, track);
    }
}

fn transition_endpoints(check: &mut Check, track: &ResolvedTrack) {
    let clips: BTreeSet<_> = track.clips.iter().map(|clip| &clip.id).collect();
    for transition in &track.transitions {
        if !clips.contains(&transition.outgoing_clip_id) {
            check.push(
                "PLAN_TRANSITION_OUTGOING_MISSING",
                Some(transition.outgoing_clip_id.to_string()),
                "transition outgoing clip is missing",
            );
        }
        if !clips.contains(&transition.incoming_clip_id) {
            check.push(
                "PLAN_TRANSITION_INCOMING_MISSING",
                Some(transition.incoming_clip_id.to_string()),
                "transition incoming clip is missing",
            );
        }
    }
}

pub(super) fn cycles(check: &mut Check, plan: &ResolvedRenderPlan) {
    let graph: BTreeMap<_, Vec<_>> = plan
        .sequences
        .iter()
        .map(|sequence| {
            let children = sequence
                .tracks
                .iter()
                .flat_map(|track| &track.clips)
                .filter_map(|clip| match &clip.source {
                    ResolvedClipSource::Sequence { sequence_id } => Some(sequence_id.clone()),
                    _ => None,
                })
                .collect();
            (sequence.id.clone(), children)
        })
        .collect();
    if cyclic(&graph) {
        check.push(
            "PLAN_SEQUENCE_CYCLE",
            Some(plan.entry_sequence_id.to_string()),
            "nested sequence graph is cyclic",
        );
    }
    let reachable = reachable(&plan.entry_sequence_id, &graph);
    if reachable.len() != graph.len() {
        check.push(
            "PLAN_SEQUENCE_UNREACHABLE",
            Some(plan.entry_sequence_id.to_string()),
            "render plan contains a sequence unreachable from its entry",
        );
    }
    let positions: BTreeMap<_, _> = plan
        .sequences
        .iter()
        .enumerate()
        .map(|(index, sequence)| (&sequence.id, index))
        .collect();
    if graph.iter().any(|(parent, children)| {
        children.iter().any(|child| {
            positions
                .get(child)
                .zip(positions.get(parent))
                .is_some_and(|(child, parent)| child >= parent)
        })
    }) {
        check.push(
            "PLAN_SEQUENCE_ORDER_INVALID",
            Some(plan.entry_sequence_id.to_string()),
            "nested sequences must precede their consumers",
        );
    }
    complexity::validate(check, plan);
}

fn reachable<T: Ord + Clone>(entry: &T, graph: &BTreeMap<T, Vec<T>>) -> BTreeSet<T> {
    let mut found = BTreeSet::new();
    let mut pending = vec![entry.clone()];
    while let Some(node) = pending.pop() {
        if found.insert(node.clone()) {
            pending.extend(graph.get(&node).into_iter().flatten().cloned());
        }
    }
    found
}

fn cyclic<T: Ord + Clone>(graph: &BTreeMap<T, Vec<T>>) -> bool {
    let mut incoming: BTreeMap<T, usize> = graph.keys().cloned().map(|id| (id, 0)).collect();
    for child in graph.values().flatten() {
        if let Some(count) = incoming.get_mut(child) {
            *count += 1;
        }
    }
    let mut pending: Vec<_> = incoming
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
        .collect();
    let mut visited = 0;
    while let Some(node) = pending.pop() {
        visited += 1;
        for child in graph.get(&node).into_iter().flatten() {
            let Some(count) = incoming.get_mut(child) else {
                continue;
            };
            *count -= 1;
            if *count == 0 {
                pending.push(child.clone());
            }
        }
    }
    visited != graph.len()
}
