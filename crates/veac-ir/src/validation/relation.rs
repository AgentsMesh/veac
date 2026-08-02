mod projection;
mod transition_contract;

use std::collections::BTreeSet;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn relations(&mut self, project: &Project) {
        let graph = RelationGraph::project(project);
        self.relation_scope(&project.sequences, &project.relations, &graph);
    }

    pub(super) fn relation_scope(
        &mut self,
        sequences: &[Sequence],
        relations: &[Relation],
        graph: &RelationGraph<'_>,
    ) {
        let mut projections = BTreeSet::new();
        for relation in relations {
            let path = format!("/project/relations/{}", relation.id);
            self.check_id(relation.id.is_valid(), relation.id.as_str(), &path);
            if !self.relation_ids.insert(relation.id.to_string()) {
                self.duplicate("DUPLICATE_RELATION_ID", relation.id.as_str(), &path);
            }
            self.check_id(
                relation.sequence_id.is_valid(),
                relation.sequence_id.as_str(),
                &format!("{path}/sequence_id"),
            );
            let Some(index) = graph.scope(&relation.sequence_id) else {
                self.missing_ref(
                    "RELATION_SEQUENCE_NOT_FOUND",
                    relation.sequence_id.as_str(),
                    &path,
                );
                continue;
            };
            let Some(projection) = self.relation_projection(relation, index, &path) else {
                continue;
            };
            if !projections.insert((relation.sequence_id.as_str(), projection)) {
                self.value_error("MULTIPLE_RELATION_PROJECTION", &path, relation.id.as_str());
            }
        }
        for sequence in sequences {
            self.composition_graph(sequence, graph);
            self.sidechain_graph(sequence, graph);
        }
        self.relation_transition_windows(sequences, graph);
    }

    pub(super) fn relation_item(
        &mut self,
        endpoint: &RelationEndpoint,
        index: &RelationSequence<'_>,
        path: &str,
        relation_id: &str,
    ) -> Option<ItemId> {
        if !self.relation_endpoint(endpoint, index, path, relation_id) {
            return None;
        }
        match endpoint {
            RelationEndpoint::Item { item_id } => Some(item_id.clone()),
            _ => {
                self.value_error("RELATION_ENDPOINT_TYPE", path, relation_id);
                None
            }
        }
    }

    pub(super) fn relation_signal(
        &mut self,
        endpoint: &RelationEndpoint,
        index: &RelationSequence<'_>,
        path: &str,
        relation_id: &str,
    ) -> Option<SidechainSource> {
        if !self.relation_endpoint(endpoint, index, path, relation_id) {
            return None;
        }
        let source = endpoint.sidechain_source();
        if source.is_none() {
            self.value_error("RELATION_ENDPOINT_TYPE", path, relation_id);
        }
        source
    }

    pub(super) fn relation_apply(
        &mut self,
        endpoint: &RelationEndpoint,
        index: &RelationSequence<'_>,
        path: &str,
        relation_id: &str,
    ) -> Option<ApplyId> {
        if !self.relation_endpoint(endpoint, index, path, relation_id) {
            return None;
        }
        match endpoint {
            RelationEndpoint::Apply { apply_id } => Some(apply_id.clone()),
            _ => {
                self.value_error("RELATION_ENDPOINT_TYPE", path, relation_id);
                None
            }
        }
    }

    fn relation_endpoint(
        &mut self,
        endpoint: &RelationEndpoint,
        index: &RelationSequence<'_>,
        path: &str,
        relation_id: &str,
    ) -> bool {
        let (valid, id) = match endpoint {
            RelationEndpoint::Item { item_id } => (item_id.is_valid(), item_id.as_str()),
            RelationEndpoint::Apply { apply_id } => (apply_id.is_valid(), apply_id.as_str()),
            RelationEndpoint::Track { track_id } => (track_id.is_valid(), track_id.as_str()),
            RelationEndpoint::Bus { bus_id } => (bus_id.is_valid(), bus_id.as_str()),
        };
        self.check_id(valid, id, path);
        if !index.contains(endpoint) {
            self.missing_ref("RELATION_ENDPOINT_NOT_FOUND", id, path);
            return false;
        }
        if !valid {
            self.value_error("RELATION_ENDPOINT_TYPE", path, relation_id);
        }
        valid
    }
}
