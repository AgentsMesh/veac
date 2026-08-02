use std::collections::{BTreeMap, BTreeSet, HashMap};

use veac_plan::canonical::{ApplyId, RationalTime, TimeRange, TrackId};
use veac_plan::{ResolvedApply, ResolvedApplyTarget, ResolvedSequence};

pub(super) struct Schedule<'a> {
    pub positions: HashMap<TrackId, usize>,
    pub due: BTreeMap<usize, Vec<&'a ResolvedApply>>,
    pub counts: HashMap<TrackId, usize>,
    pub checkpoints: HashMap<TrackId, Vec<(TrackId, usize)>>,
    pub propagation: HashMap<ApplyId, Vec<TrackId>>,
}

type Checkpoints = HashMap<TrackId, Vec<(TrackId, usize)>>;
type Propagation = HashMap<ApplyId, Vec<TrackId>>;

struct Scheduled<'a> {
    apply: &'a ResolvedApply,
    from: usize,
    through: usize,
    target: TrackId,
}

pub(super) fn build(sequence: &ResolvedSequence) -> Schedule<'_> {
    let positions: HashMap<_, _> = sequence
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| (track.id.clone(), index))
        .collect();
    let mut scheduled: Vec<_> = sequence
        .applies
        .iter()
        .filter_map(|apply| {
            interval(apply, &positions).map(|(from, through, target)| Scheduled {
                apply,
                from,
                through,
                target,
            })
        })
        .collect();
    scheduled.sort_by_key(|entry| {
        (
            entry.through,
            std::cmp::Reverse(entry.from),
            entry.apply.source_order,
        )
    });
    let mut counts = HashMap::new();
    let mut due: BTreeMap<usize, Vec<&ResolvedApply>> = BTreeMap::new();
    for entry in &scheduled {
        *counts.entry(entry.target.clone()).or_insert(0) += 1;
        due.entry(entry.through).or_default().push(entry.apply);
    }
    let (checkpoints, propagation) = dependencies(&scheduled);
    Schedule {
        positions,
        due,
        counts,
        checkpoints,
        propagation,
    }
}

fn dependencies(scheduled: &[Scheduled<'_>]) -> (Checkpoints, Propagation) {
    let mut uses: HashMap<(TrackId, TrackId), usize> = HashMap::new();
    let mut propagation = HashMap::new();
    for (index, nested) in scheduled.iter().enumerate() {
        let broader: BTreeSet<_> = scheduled[index + 1..]
            .iter()
            .filter(|candidate| contains(candidate, nested))
            .map(|candidate| candidate.target.clone())
            .collect();
        for target in &broader {
            *uses
                .entry((nested.target.clone(), target.clone()))
                .or_insert(0) += 1;
        }
        if !broader.is_empty() {
            propagation.insert(nested.apply.id.clone(), broader.into_iter().collect());
        }
    }
    let mut checkpoints: HashMap<_, Vec<_>> = HashMap::new();
    for ((nested, broader), count) in uses {
        checkpoints
            .entry(nested)
            .or_default()
            .push((broader, count));
    }
    (checkpoints, propagation)
}

fn contains(broader: &Scheduled<'_>, nested: &Scheduled<'_>) -> bool {
    broader.from < nested.from && broader.through >= nested.through
}

pub(super) fn target_id(apply: &ResolvedApply) -> Option<TrackId> {
    match &apply.target {
        ResolvedApplyTarget::CompositeBand { from_track_id, .. } => Some(from_track_id.clone()),
        ResolvedApplyTarget::Layer { track_id, .. } => Some(track_id.clone()),
        ResolvedApplyTarget::ItemSet { .. } => None,
    }
}

pub(super) fn full_range(sequence: &ResolvedSequence) -> TimeRange {
    TimeRange {
        start: RationalTime {
            value: 0,
            timescale: sequence.duration.timescale,
        },
        duration: sequence.duration,
    }
}

fn interval(
    apply: &ResolvedApply,
    positions: &HashMap<TrackId, usize>,
) -> Option<(usize, usize, TrackId)> {
    let (from, through) = match &apply.target {
        ResolvedApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
            ..
        } => (from_track_id, through_track_id),
        ResolvedApplyTarget::Layer { track_id, .. } => (track_id, track_id),
        ResolvedApplyTarget::ItemSet { .. } => return None,
    };
    Some((
        *positions.get(from)?,
        *positions.get(through)?,
        from.clone(),
    ))
}
