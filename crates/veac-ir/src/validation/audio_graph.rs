use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn sidechain_graph(&mut self, sequence: &Sequence, relations: &RelationGraph<'_>) {
        let activity = SequenceActivity::new(sequence);
        for edge in relations.sidechains(&sequence.id) {
            if !activity.audio_live(edge.target.track, edge.target.clip) {
                continue;
            }
            let path = format!("/project/relations/{}/kind/key", edge.relation_id);
            let (typed, self_dependency, live) = match &edge.key {
                RelationSignal::Track(source) => (
                    track_audio_typed(&activity, source),
                    source.id == edge.target.track.id,
                    activity.track_has_live_audio(source),
                ),
                RelationSignal::Bus { tracks, .. } => (
                    tracks
                        .iter()
                        .any(|track| track_audio_typed(&activity, track)),
                    tracks.iter().any(|track| track.id == edge.target.track.id),
                    tracks
                        .iter()
                        .any(|track| activity.track_has_live_audio(track)),
                ),
            };
            let code = if !typed {
                Some("SIDECHAIN_SOURCE_TYPE")
            } else if self_dependency {
                Some("SIDECHAIN_SELF_DEPENDENCY")
            } else if !live {
                Some("SIDECHAIN_SOURCE_INACTIVE")
            } else {
                None
            };
            if let Some(code) = code {
                self.push(
                    code,
                    Some(edge.relation_id.to_string()),
                    path,
                    "sidechain key must be another active audio-producing signal",
                    None,
                );
            }
        }
    }
}

fn track_audio_typed(activity: &SequenceActivity<'_>, track: &Track) -> bool {
    track
        .clips
        .iter()
        .any(|clip| activity.audio_typed(track, clip))
}
