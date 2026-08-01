use veac_plan::canonical::{SidechainSource, TrackKind};
use veac_plan::{
    ResolvedAudioRoute, ResolvedClip, ResolvedSequence, ResolvedSidechain, ResolvedTrack,
};

use super::audio::AudioRenderSpec;
use super::error::{diagnostic, CodegenErrorKind};
use super::{
    audio, audio_filters, audio_source, audio_transition_fades, time, CodegenErrors, EmitContext,
};

#[cfg(test)]
#[path = "../unit_tests/audio_sidechain_internal_tests.rs"]
mod internal_tests;

mod range;

pub(super) fn source(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    target_track: &ResolvedTrack,
    target: &ResolvedClip,
    value: &ResolvedSidechain,
    output: &AudioRenderSpec,
) -> Result<Option<String>, CodegenErrors> {
    let active = active_window(target, value).ok_or_else(|| {
        unsupported(
            target,
            "sidechain active range overflows the sequence timeline",
        )
    })?;
    let tracks: Vec<_> = sequence
        .tracks
        .iter()
        .filter(|track| source_track(track, target_track, &value.source))
        .collect();
    let mut labels = Vec::new();
    for track in tracks {
        for clip in track
            .clips
            .iter()
            .filter(|clip| intersects(clip.record_range, active))
        {
            let fades = transition_fades(track, clip);
            if let Some(label) = audio_source::build(context, clip, output, fades, None)? {
                labels.push(label);
            }
        }
    }
    if labels.is_empty() {
        return Ok(None);
    }
    let mixed = audio::mix(context, &labels, "sidechainsourcemix");
    Ok(Some(context.graph.filter(
        &[&mixed],
        format!(
            "atrim=start={}:duration={},asetpts=PTS-STARTPTS,apad=whole_dur={},atrim=duration={}",
            time::seconds(target.record_range.start),
            time::seconds(target.record_range.duration),
            time::seconds(target.record_range.duration),
            time::seconds(target.record_range.duration)
        ),
        "sidechainsourcea",
    )))
}

pub(super) fn active_window(
    clip: &ResolvedClip,
    value: &ResolvedSidechain,
) -> Option<veac_plan::canonical::TimeRange> {
    let Some(local) = value.active_range else {
        return Some(clip.record_range);
    };
    let start = clip.record_range.start.checked_add(local.start).ok()?;
    Some(veac_plan::canonical::TimeRange {
        start,
        duration: local.duration,
    })
}

fn intersects(
    left: veac_plan::canonical::TimeRange,
    right: veac_plan::canonical::TimeRange,
) -> bool {
    left.end()
        .ok()
        .zip(right.end().ok())
        .is_some_and(|(left_end, right_end)| left.start < right_end && right.start < left_end)
}

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input: String,
    sidechain: String,
    value: &ResolvedSidechain,
) -> Result<String, CodegenErrors> {
    if let Some(active) = value.active_range {
        if active.start.value != 0 || active.duration != clip.record_range.duration {
            return range::apply(context, clip, &input, &sidechain, value, active);
        }
    }
    Ok(compress(context, &input, &sidechain, value))
}

pub(super) fn compress(
    context: &mut EmitContext<'_>,
    input: &str,
    sidechain: &str,
    value: &ResolvedSidechain,
) -> String {
    context.graph.filter(
        &[input, sidechain],
        format!(
            "sidechaincompress=threshold={}:ratio={}:attack={}:release={}:mix=1",
            audio_filters::amplitude(value.threshold_db),
            time::number(value.ratio),
            time::number(value.attack_ms),
            time::number(value.release_ms)
        ),
        "sidechaina",
    )
}

fn source_track(track: &ResolvedTrack, target: &ResolvedTrack, source: &SidechainSource) -> bool {
    if track.id == target.id
        || !track.state.audio_enabled
        || !matches!(track.kind, TrackKind::Video | TrackKind::Audio)
    {
        return false;
    }
    match source {
        SidechainSource::Track { track_id } => track.id == *track_id,
        SidechainSource::Bus { bus_id } => {
            matches!(&track.routing.audio, Some(ResolvedAudioRoute::Bus { bus_id: source }) if source == bus_id.as_str())
        }
    }
}

fn transition_fades(
    track: &ResolvedTrack,
    clip: &ResolvedClip,
) -> audio_transition_fades::TransitionFades {
    audio_transition_fades::TransitionFades {
        fade_in: track
            .transitions
            .iter()
            .find(|value| value.incoming_clip_id == clip.id)
            .map(|value| value.incoming_handle.duration),
        fade_out: track
            .transitions
            .iter()
            .find(|value| value.outgoing_clip_id == clip.id)
            .map(|value| value.outgoing_handle.duration),
    }
}

fn unsupported(clip: &ResolvedClip, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::UnsupportedAudioProcessing,
        "AUDIO_PROCESSING_UNSUPPORTED",
        Some(clip.id.to_string()),
        message,
    ))
}
