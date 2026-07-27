use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn sidechain_graph(&mut self, sequence: &Sequence, relations: &RelationGraph<'_>) {
        for edge in relations.sidechains(&sequence.id) {
            let path = format!("/project/relations/{}/kind/key", edge.relation_id);
            let valid = match &edge.key {
                RelationSignal::Track(source) => {
                    source.id != edge.target.track.id && audio_track(source)
                }
                RelationSignal::Bus { tracks, .. } => {
                    !tracks.iter().any(|track| track.id == edge.target.track.id)
                        && tracks.iter().any(|track| audio_track(track))
                }
            };
            if !valid {
                let code = match edge.key {
                    RelationSignal::Track(_) => "SIDECHAIN_SOURCE_TRACK",
                    RelationSignal::Bus { .. } => "SIDECHAIN_SOURCE_BUS",
                };
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

fn audio_track(track: &Track) -> bool {
    track.state.enabled
        && !track.state.muted
        && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
        && track.clips.iter().any(audio_producing)
}

fn audio_producing(clip: &Clip) -> bool {
    clip.enabled && clip.audio.as_ref().is_some_and(|audio| !audio.muted)
}
