use veac_ir::{AudioStemSource, DeliverableKind, RenderConfig, Track, TrackKind, TrackRouting};

use crate::{EffectiveTrackState, ResolvedAudioRoute, ResolvedTrackRouting, ResolvedVisualRoute};

pub(super) fn effective(
    track: &Track,
    has_solo: bool,
    config: &RenderConfig,
) -> EffectiveTrackState {
    let active = track.state.enabled && (!has_solo || track.state.solo);
    let visual_enabled = active
        && match track.kind {
            TrackKind::Video | TrackKind::Visual => output_visual(config),
            TrackKind::Caption => burn_captions(config),
            TrackKind::Audio => false,
        };
    let audio_enabled = active
        && output_audio(track, config)
        && !track.state.muted
        && matches!(track.kind, TrackKind::Video | TrackKind::Audio);
    EffectiveTrackState {
        include_in_render: visual_enabled
            || audio_enabled
            || (active && track.kind == TrackKind::Caption && sidecar_captions(track, config)),
        visual_enabled,
        audio_enabled,
    }
}

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

fn output_visual(config: &RenderConfig) -> bool {
    config.deliverables.iter().any(|value| {
        matches!(
            value.kind,
            DeliverableKind::Video(_)
                | DeliverableKind::ImageSequence(_)
                | DeliverableKind::Scope(_)
        )
    })
}

fn output_audio(track: &Track, config: &RenderConfig) -> bool {
    config.deliverables.iter().any(|value| match &value.kind {
        DeliverableKind::Video(settings) => settings.audio.is_some(),
        DeliverableKind::AudioStem(settings) => match &settings.source {
            AudioStemSource::Master => true,
            AudioStemSource::Track { track_id } => *track_id == track.id,
            AudioStemSource::Bus { bus_id } => matches!(
                &track.routing,
                TrackRouting::AudioBus {
                    bus_id: track_bus_id
                } if track_bus_id == bus_id
            ),
        },
        _ => false,
    })
}

fn burn_captions(config: &RenderConfig) -> bool {
    config.deliverables.iter().any(|value| {
        matches!(
            &value.kind,
            DeliverableKind::Video(settings)
                if settings.captions == veac_ir::CaptionOutput::BurnIn
        )
    })
}

fn sidecar_captions(track: &Track, config: &RenderConfig) -> bool {
    config.deliverables.iter().any(|value| {
        matches!(
            &value.kind,
            DeliverableKind::CaptionSidecar(settings)
                if settings.track_ids.contains(&track.id)
        )
    })
}
