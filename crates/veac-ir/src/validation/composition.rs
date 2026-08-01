use crate::*;

use super::Validator;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MatteNode {
    Item(ItemId),
    Apply(ApplyId),
}

impl Validator {
    pub(super) fn composition_graph(&mut self, sequence: &Sequence, relations: &RelationGraph<'_>) {
        let activity = SequenceActivity::new(sequence);
        let mut graph = MatteDependencyGraph::new();
        for edge in relations.mattes(&sequence.id) {
            self.clip_matte(&activity, edge, &mut graph);
        }
        for edge in relations.apply_mattes(&sequence.id) {
            self.apply_matte(&activity, edge, &mut graph);
        }
        let analysis = graph.analyze();
        if analysis.cyclic {
            self.value_error("MATTE_CYCLE", "/project/relations", sequence.id.as_str());
        }
        if analysis.max_depth > MAX_MATTE_NESTING_DEPTH {
            self.value_error(
                "MATTE_DEPTH_EXCEEDED",
                "/project/relations",
                sequence.id.as_str(),
            );
        }
    }

    fn clip_matte(
        &mut self,
        activity: &SequenceActivity<'_>,
        edge: MatteEdge<'_>,
        graph: &mut MatteDependencyGraph<MatteNode>,
    ) {
        let path = format!("/project/relations/{}/kind", edge.relation_id);
        let consumer_typed = activity.visual_typed(edge.consumer.track, edge.consumer.clip);
        let source_typed = activity.visual_typed(edge.producer.track, edge.producer.clip);
        if !consumer_typed {
            self.value_error("MATTE_CONSUMER_TYPE", &path, edge.relation_id.as_str());
        }
        if !source_typed {
            self.value_error("MATTE_SOURCE_TYPE", &path, edge.relation_id.as_str());
        }
        let consumer_live =
            consumer_typed && activity.visual_live(edge.consumer.track, edge.consumer.clip);
        let source_live =
            source_typed && activity.visual_live(edge.producer.track, edge.producer.clip);
        let self_dependency = edge.consumer.clip.id == edge.producer.clip.id;
        let range_valid = covers(
            edge.producer.clip.record_range,
            edge.consumer.clip.record_range,
        );
        self.runtime_matte(
            consumer_live && source_typed,
            source_live,
            range_valid,
            &path,
            edge.relation_id,
        );
        if consumer_live && source_live && range_valid && !self_dependency {
            graph.add(
                MatteNode::Item(edge.consumer.clip.id.clone()),
                MatteNode::Item(edge.producer.clip.id.clone()),
            );
        }
    }

    fn apply_matte(
        &mut self,
        activity: &SequenceActivity<'_>,
        edge: ApplyMatteEdge<'_>,
        graph: &mut MatteDependencyGraph<MatteNode>,
    ) {
        let path = format!("/project/relations/{}/kind", edge.relation_id);
        let source_typed = activity.visual_typed(edge.producer.track, edge.producer.clip);
        if !source_typed {
            self.value_error("MATTE_SOURCE_TYPE", &path, edge.relation_id.as_str());
        }
        let targets = activity.live_apply_targets(edge.consumer.apply);
        let consumer_live = !targets.is_empty();
        let source_live =
            source_typed && activity.visual_live(edge.producer.track, edge.producer.clip);
        let range_valid = covers(
            edge.producer.clip.record_range,
            edge.consumer.apply.record_range,
        );
        self.runtime_matte(
            consumer_live && source_typed,
            source_live,
            range_valid,
            &path,
            edge.relation_id,
        );
        let self_dependency = super::apply::target_contains(
            edge.consumer.sequence,
            &edge.consumer.apply.target,
            edge.producer,
        );
        if self_dependency {
            self.value_error(
                "APPLY_MATTE_SELF_DEPENDENCY",
                &path,
                edge.relation_id.as_str(),
            );
        }
        if !consumer_live || !source_live || !range_valid || self_dependency {
            return;
        }
        let producer = MatteNode::Item(edge.producer.clip.id.clone());
        match &edge.consumer.apply.target {
            ApplyTarget::ItemSet { .. } => {
                for target in targets {
                    graph.add(MatteNode::Item(target.clip.id.clone()), producer.clone());
                }
            }
            ApplyTarget::Layer { .. } | ApplyTarget::CompositeBand { .. } => {
                graph.add(MatteNode::Apply(edge.consumer.apply.id.clone()), producer)
            }
        }
    }

    fn runtime_matte(
        &mut self,
        consumer_live: bool,
        source_live: bool,
        range_valid: bool,
        path: &str,
        relation_id: &RelationId,
    ) {
        if consumer_live && !source_live {
            self.value_error("MATTE_SOURCE_INACTIVE", path, relation_id.as_str());
        } else if consumer_live && !range_valid {
            self.value_error("MATTE_SOURCE_RANGE", path, relation_id.as_str());
        }
    }
}

fn covers(outer: TimeRange, inner: TimeRange) -> bool {
    outer.start <= inner.start
        && outer
            .end()
            .is_ok_and(|end| inner.end().is_ok_and(|inner_end| end >= inner_end))
}
