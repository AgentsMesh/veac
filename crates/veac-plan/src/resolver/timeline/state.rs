use veac_ir::{Track, TrackRouting};

use crate::{EffectiveTrackState, ResolvedAudioRoute, ResolvedTrackRouting, ResolvedVisualRoute};

pub(super) fn routing(track: &Track, state: EffectiveTrackState) -> ResolvedTrackRouting {
    let visual = state
        .visual_enabled
        .then_some(ResolvedVisualRoute::MainComposite);
    let audio = state.audio_enabled.then(|| match &track.routing {
        TrackRouting::Default => ResolvedAudioRoute::MainMix,
        TrackRouting::AudioBus { bus_id } => ResolvedAudioRoute::Bus {
            bus_id: bus_id.to_string(),
        },
    });
    ResolvedTrackRouting { visual, audio }
}
