use std::collections::BTreeSet;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn group_relation(
        &mut self,
        members: &[RelationEndpoint],
        index: &RelationSequence<'_>,
        path: &str,
        id: &RelationId,
    ) -> Option<String> {
        let members = self.item_members(members, index, &format!("{path}/kind/members"), id, 2)?;
        for member in members {
            if !self.relation_group_members.insert(member.to_string()) {
                self.value_error("MULTIPLE_GROUP_MEMBERSHIP", path, member.as_str());
            }
        }
        Some(format!("group:{id}"))
    }

    pub(super) fn av_link_relation(
        &mut self,
        video: &RelationEndpoint,
        audio: &[RelationEndpoint],
        index: &RelationSequence<'_>,
        path: &str,
        id: &RelationId,
    ) -> Option<String> {
        let video_id =
            self.relation_item(video, index, &format!("{path}/kind/video"), id.as_str())?;
        let video_item = index.item(&video_id)?;
        if !matches!(video_item.track.kind, TrackKind::Video | TrackKind::Visual) {
            self.value_error("AV_LINK_VIDEO", path, video_id.as_str());
        }
        let audio = self.item_members(audio, index, &format!("{path}/kind/audio"), id, 1)?;
        self.claim_av_member(&video_id, path);
        for audio_id in audio {
            let audio_item = index.item(&audio_id)?;
            if audio_item.track.kind != TrackKind::Audio {
                self.value_error("AV_LINK_AUDIO", path, audio_id.as_str());
            } else if video_item.clip.record_range != audio_item.clip.record_range {
                self.value_error("AV_LINK_SYNC", path, audio_id.as_str());
            }
            self.claim_av_member(&audio_id, path);
        }
        Some(format!("av_link:{id}"))
    }

    fn item_members(
        &mut self,
        endpoints: &[RelationEndpoint],
        index: &RelationSequence<'_>,
        path: &str,
        id: &RelationId,
        minimum: usize,
    ) -> Option<Vec<ItemId>> {
        if endpoints.len() < minimum {
            self.value_error("RELATION_MEMBERS", path, id.as_str());
            return None;
        }
        let mut members = Vec::with_capacity(endpoints.len());
        let mut unique = BTreeSet::new();
        for (position, endpoint) in endpoints.iter().enumerate() {
            let member =
                self.relation_item(endpoint, index, &format!("{path}/{position}"), id.as_str())?;
            if !unique.insert(member.clone()) {
                self.duplicate("DUPLICATE_RELATION_MEMBER", member.as_str(), path);
                return None;
            }
            members.push(member);
        }
        Some(members)
    }

    fn claim_av_member(&mut self, member: &ItemId, path: &str) {
        if !self.relation_av_link_members.insert(member.to_string()) {
            self.value_error("MULTIPLE_AV_LINK_MEMBERSHIP", path, member.as_str());
        }
    }
}
