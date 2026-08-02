use std::collections::BTreeMap;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn sequence_cycles(&mut self, sequences: &[Sequence]) {
        let adjacency: BTreeMap<_, Vec<_>> = sequences
            .iter()
            .map(|sequence| {
                let nested = sequence
                    .tracks
                    .iter()
                    .flat_map(|track| &track.clips)
                    .filter_map(|clip| match &clip.source {
                        ClipSource::Sequence { sequence_id } => Some(sequence_id.as_str()),
                        _ => None,
                    })
                    .collect();
                (sequence.id.as_str(), nested)
            })
            .collect();
        let (cyclic, depth) = graph_shape(&adjacency);
        if cyclic {
            self.value_error("SEQUENCE_CYCLE", "/project/sequences", "sequences");
        }
        if depth > MAX_SEQUENCE_NESTING_DEPTH {
            self.value_error("SEQUENCE_DEPTH", "/project/sequences", "sequences");
        }
    }
}

fn graph_shape<'a>(adjacency: &BTreeMap<&'a str, Vec<&'a str>>) -> (bool, usize) {
    let mut incoming: BTreeMap<_, usize> = adjacency.keys().map(|id| (*id, 0)).collect();
    for child in adjacency.values().flatten() {
        if let Some(count) = incoming.get_mut(child) {
            *count += 1;
        }
    }
    let mut depths: BTreeMap<_, usize> = adjacency.keys().map(|id| (*id, 1)).collect();
    let mut pending: Vec<_> = incoming
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect();
    let mut visited = 0;
    while let Some(id) = pending.pop() {
        visited += 1;
        let parent_depth = depths[&id];
        for child in adjacency.get(id).into_iter().flatten() {
            let Some(count) = incoming.get_mut(child) else {
                continue;
            };
            depths
                .entry(child)
                .and_modify(|depth| *depth = (*depth).max(parent_depth + 1));
            *count -= 1;
            if *count == 0 {
                pending.push(child);
            }
        }
    }
    (
        visited != adjacency.len(),
        depths.into_values().max().unwrap_or(0),
    )
}
