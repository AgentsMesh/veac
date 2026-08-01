mod membership;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn relation_projection<'a>(
        &mut self,
        relation: &Relation,
        index: &'a RelationSequence<'a>,
        path: &str,
    ) -> Option<String> {
        match &relation.kind {
            RelationKind::Transition {
                from,
                to,
                transition,
            } => self.transition_relation(from, to, transition, index, path, &relation.id),
            RelationKind::Matte {
                producer,
                consumer,
                parameters,
            } => self.matte_relation(producer, consumer, parameters, index, path, &relation.id),
            RelationKind::Sidechain {
                key,
                target,
                parameters,
            } => self.sidechain_relation(key, target, parameters, index, path, &relation.id),
            RelationKind::Group { members } => {
                self.group_relation(members, index, path, &relation.id)
            }
            RelationKind::AvLink { video, audio } => {
                self.av_link_relation(video, audio, index, path, &relation.id)
            }
        }
    }

    fn transition_relation(
        &mut self,
        from: &RelationEndpoint,
        to: &RelationEndpoint,
        transition: &Transition,
        index: &RelationSequence<'_>,
        path: &str,
        id: &RelationId,
    ) -> Option<String> {
        let from_id = self.relation_item(from, index, &format!("{path}/kind/from"), id.as_str())?;
        let to_id = self.relation_item(to, index, &format!("{path}/kind/to"), id.as_str())?;
        let from_item = index.item(&from_id)?;
        let to_item = index.item(&to_id)?;
        let adjacent = from_item.track.id == to_item.track.id
            && from_item
                .track
                .clips
                .get(from_item.position + 1)
                .is_some_and(|clip| clip.id == to_id);
        if !adjacent {
            self.value_error("RELATION_CONTEXT", path, id.as_str());
            return None;
        }
        self.transition_contract(transition, from_item, to_item, path, id);
        Some(format!("transition:{from_id}"))
    }

    fn matte_relation(
        &mut self,
        producer: &RelationEndpoint,
        consumer: &RelationEndpoint,
        parameters: &MatteRelationParameters,
        index: &RelationSequence<'_>,
        path: &str,
        id: &RelationId,
    ) -> Option<String> {
        let producer = self.relation_item(
            producer,
            index,
            &format!("{path}/kind/producer"),
            id.as_str(),
        )?;
        let consumer_path = format!("{path}/kind/consumer");
        let projection = match consumer {
            RelationEndpoint::Item { .. } => {
                let consumer = self.relation_item(consumer, index, &consumer_path, id.as_str())?;
                if producer == consumer {
                    self.value_error("RELATION_CONTEXT", path, id.as_str());
                }
                format!("matte:item:{consumer}")
            }
            RelationEndpoint::Apply { .. } => {
                let consumer = self.relation_apply(consumer, index, &consumer_path, id.as_str())?;
                format!("matte:apply:{consumer}")
            }
            RelationEndpoint::Track { .. } | RelationEndpoint::Bus { .. } => {
                self.relation_endpoint(consumer, index, &consumer_path, id.as_str());
                self.value_error("RELATION_ENDPOINT_TYPE", &consumer_path, id.as_str());
                return None;
            }
        };
        let _ = parameters;
        Some(projection)
    }

    fn sidechain_relation(
        &mut self,
        key: &RelationEndpoint,
        target: &RelationEndpoint,
        parameters: &SidechainRelationParameters,
        index: &RelationSequence<'_>,
        path: &str,
        id: &RelationId,
    ) -> Option<String> {
        self.relation_signal(key, index, &format!("{path}/kind/key"), id.as_str())?;
        let target =
            self.relation_item(target, index, &format!("{path}/kind/target"), id.as_str())?;
        let target_item = index.item(&target)?;
        let activity = SequenceActivity::new(index.sequence());
        if !activity.audio_typed(target_item.track, target_item.clip) {
            self.value_error(
                "SIDECHAIN_TARGET_TYPE",
                &format!("{path}/kind/target"),
                id.as_str(),
            );
        }
        self.sidechain(
            parameters,
            target_item.clip.record_range.duration,
            target_item.clip.record_range.duration.timescale,
            path,
            id.as_str(),
        );
        Some(format!("sidechain:{target}"))
    }
}
