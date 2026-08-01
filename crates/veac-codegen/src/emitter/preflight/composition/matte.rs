use std::collections::BTreeMap;

use veac_plan::canonical::{
    ApplyId, ItemId, MatteDependencyGraph, TimeRange, MAX_MATTE_NESTING_DEPTH,
};
use veac_plan::{
    ResolvedApply, ResolvedApplyTarget, ResolvedClip, ResolvedSequence, ResolvedTrack,
};

use super::super::{apply_target, Check};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum MatteNode {
    Item(ItemId),
    Apply(ApplyId),
}

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence) {
    let clips: BTreeMap<_, _> = sequence
        .tracks
        .iter()
        .flat_map(|track| {
            track
                .clips
                .iter()
                .map(move |clip| (&clip.id, (track, clip)))
        })
        .collect();
    let mut graph = MatteDependencyGraph::new();
    for (track, clip) in clips.values().copied() {
        clip_matte(check, track, clip, &clips, &mut graph);
    }
    for apply in &sequence.applies {
        apply_matte(check, apply, &clips, &mut graph);
    }
    let analysis = graph.analyze();
    if analysis.cyclic {
        check.push(
            "PLAN_MATTE_CYCLE",
            Some(sequence.id.to_string()),
            "clip and apply matte references are cyclic",
        );
    }
    if analysis.max_depth > MAX_MATTE_NESTING_DEPTH {
        check.push(
            "PLAN_MATTE_DEPTH_EXCEEDED",
            Some(sequence.id.to_string()),
            "matte dependency depth exceeds the executable limit",
        );
    }
}

fn clip_matte(
    check: &mut Check,
    track: &ResolvedTrack,
    clip: &ResolvedClip,
    clips: &ClipIndex<'_>,
    graph: &mut MatteDependencyGraph<MatteNode>,
) {
    let Some(matte) = clip
        .visual
        .as_ref()
        .filter(|_| track.state.visual_enabled)
        .and_then(|visual| visual.track_matte.as_ref())
    else {
        return;
    };
    let source = clips.get(&matte.source_clip_id).copied();
    let valid = source.is_some_and(|(source_track, source_clip)| {
        source_clip.id != clip.id
            && live(source_track, source_clip)
            && covers(source_clip.record_range, clip.record_range)
    });
    if !valid {
        check.push(
            "PLAN_MATTE_REFERENCE_INVALID",
            Some(clip.id.to_string()),
            "track matte source is missing, inactive, self-referential, or too short",
        );
        return;
    }
    graph.add(
        MatteNode::Item(clip.id.clone()),
        MatteNode::Item(matte.source_clip_id.clone()),
    );
}

fn apply_matte(
    check: &mut Check,
    apply: &ResolvedApply,
    clips: &ClipIndex<'_>,
    graph: &mut MatteDependencyGraph<MatteNode>,
) {
    let Some(matte) = &apply.matte else {
        return;
    };
    let source = clips.get(&matte.source_clip_id).copied();
    let valid = source.is_some_and(|(track, clip)| {
        live(track, clip)
            && covers(clip.record_range, apply.record_range)
            && !apply_target::contains_item(apply, &track.id, &clip.id)
    });
    if !valid {
        check.push(
            "PLAN_APPLY_MATTE_INVALID",
            Some(apply.id.to_string()),
            "apply matte source is missing, inactive, too short, or part of its target",
        );
        return;
    }
    let producer = MatteNode::Item(matte.source_clip_id.clone());
    match &apply.target {
        ResolvedApplyTarget::ItemSet { items } => {
            for item in items
                .iter()
                .filter(|item| crate::emitter::apply::target_used(apply, item.active_range))
            {
                graph.add(MatteNode::Item(item.item_id.clone()), producer.clone());
            }
        }
        ResolvedApplyTarget::Layer { .. } | ResolvedApplyTarget::CompositeBand { .. } => {
            if apply
                .stages
                .iter()
                .any(|stage| crate::emitter::apply::stage_used(apply, stage))
            {
                graph.add(MatteNode::Apply(apply.id.clone()), producer);
            }
        }
    }
}

type ClipIndex<'a> = BTreeMap<&'a ItemId, (&'a ResolvedTrack, &'a ResolvedClip)>;

fn live(track: &ResolvedTrack, clip: &ResolvedClip) -> bool {
    track.state.visual_enabled && clip.visual.is_some()
}

fn covers(outer: TimeRange, inner: TimeRange) -> bool {
    outer.start <= inner.start
        && outer
            .end()
            .is_ok_and(|end| inner.end().is_ok_and(|inner_end| end >= inner_end))
}
