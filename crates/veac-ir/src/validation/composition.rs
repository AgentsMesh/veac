use std::collections::{BTreeMap, BTreeSet};

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn composition_graph(&mut self, sequence: &Sequence, relations: &RelationGraph<'_>) {
        let has_solo = sequence
            .tracks
            .iter()
            .any(|track| track.state.enabled && track.state.solo);
        let mut graph = BTreeMap::new();
        for edge in relations.mattes(&sequence.id) {
            let path = format!("/project/relations/{}/kind", edge.relation_id);
            graph.insert(
                edge.consumer.clip.id.as_str(),
                edge.producer.clip.id.as_str(),
            );
            if !valid_source(edge.producer, edge.consumer.track, has_solo) {
                self.value_error("MATTE_SOURCE_TYPE", &path, edge.relation_id.as_str());
            } else if !covers(
                edge.producer.clip.record_range,
                edge.consumer.clip.record_range,
            ) {
                self.value_error("MATTE_SOURCE_RANGE", &path, edge.relation_id.as_str());
            }
        }
        for edge in relations.apply_mattes(&sequence.id) {
            let path = format!("/project/relations/{}/kind", edge.relation_id);
            if !valid_apply_source(edge.producer) {
                self.value_error("MATTE_SOURCE_TYPE", &path, edge.relation_id.as_str());
            } else if !covers(
                edge.producer.clip.record_range,
                edge.consumer.apply.record_range,
            ) {
                self.value_error("MATTE_SOURCE_RANGE", &path, edge.relation_id.as_str());
            }
            if super::apply::target_contains(sequence, &edge.consumer.apply.target, edge.producer) {
                self.value_error(
                    "APPLY_MATTE_SELF_DEPENDENCY",
                    &path,
                    edge.relation_id.as_str(),
                );
            }
        }
        if cyclic(&graph) {
            self.value_error("MATTE_CYCLE", "/project/relations", sequence.id.as_str());
        }
        if depth_exceeded(&graph) {
            self.value_error(
                "MATTE_DEPTH_EXCEEDED",
                "/project/relations",
                sequence.id.as_str(),
            );
        }
    }
}

fn valid_apply_source(source: RelationItem<'_>) -> bool {
    source.clip.enabled
        && source.clip.visual.is_some()
        && source.track.state.enabled
        && matches!(
            source.track.kind,
            TrackKind::Video | TrackKind::Visual | TrackKind::Caption
        )
}

fn valid_source(source: RelationItem<'_>, target: &Track, has_solo: bool) -> bool {
    source.clip.enabled
        && source.clip.visual.is_some()
        && source.track.state.enabled
        && (!has_solo || source.track.state.solo == target.state.solo)
        && matches!(
            source.track.kind,
            TrackKind::Video | TrackKind::Visual | TrackKind::Caption
        )
}

fn covers(outer: TimeRange, inner: TimeRange) -> bool {
    outer.start <= inner.start
        && outer
            .end()
            .is_ok_and(|end| inner.end().is_ok_and(|inner_end| end >= inner_end))
}

fn cyclic(graph: &BTreeMap<&str, &str>) -> bool {
    graph.keys().any(|start| {
        let mut seen = BTreeSet::new();
        let mut current = *start;
        while seen.insert(current) {
            let Some(next) = graph.get(current) else {
                return false;
            };
            current = next;
        }
        true
    })
}

fn depth_exceeded(graph: &BTreeMap<&str, &str>) -> bool {
    graph.keys().any(|start| {
        let mut seen = BTreeSet::new();
        let mut current = *start;
        let mut depth = 0;
        while seen.insert(current) {
            let Some(next) = graph.get(current) else {
                return false;
            };
            depth += 1;
            if depth > MAX_MATTE_NESTING_DEPTH {
                return true;
            }
            current = next;
        }
        false
    })
}
