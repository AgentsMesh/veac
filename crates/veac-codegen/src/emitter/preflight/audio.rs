use veac_plan::canonical::{
    audio_processor_valid, AudioProcessorKind, RationalTime, SidechainSource, TimeRange, TrackKind,
};
use veac_plan::{
    EffectiveAudioProperties, ResolvedClip, ResolvedSequence, ResolvedSidechain, ResolvedTrack,
};

use super::Check;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence, timebase: u32) {
    for track in &sequence.tracks {
        for clip in &track.clips {
            let Some(audio) = &clip.audio else { continue };
            if !processors_valid(audio, sequence.settings.sample_rate) {
                invalid(
                    check,
                    clip,
                    "audio processor chain is outside its executable domain",
                );
            }
            if audio.crossfade.is_some_and(|value| {
                !crossfade_valid(
                    value.fade_in,
                    value.fade_out,
                    clip.record_range.duration,
                    timebase,
                )
            }) {
                invalid(
                    check,
                    clip,
                    "audio crossfade is outside the clip-local timeline",
                );
            }
            if audio.sidechain.as_ref().is_some_and(|value| {
                !sidechain_valid(value, sequence, track, clip.record_range.duration, timebase)
            }) {
                invalid(
                    check,
                    clip,
                    "sidechain parameters or source routing are invalid",
                );
            }
        }
    }
}

fn processors_valid(audio: &EffectiveAudioProperties, sample_rate: u32) -> bool {
    let loudness = audio
        .processors
        .iter()
        .filter(|value| matches!(value.kind, AudioProcessorKind::Loudness(_)))
        .count();
    audio.processors.len() <= 64
        && loudness <= 1
        && !(audio.normalize && loudness > 0)
        && audio
            .processors
            .iter()
            .all(|value| audio_processor_valid(value, sample_rate))
}

fn crossfade_valid(
    fade_in: RationalTime,
    fade_out: RationalTime,
    duration: RationalTime,
    timebase: u32,
) -> bool {
    [fade_in, fade_out, duration]
        .into_iter()
        .all(|value| value.is_valid() && value.timescale == timebase)
        && fade_in.value >= 0
        && fade_out.value >= 0
        && duration.value > 0
        && fade_in
            .value
            .checked_add(fade_out.value)
            .is_some_and(|total| total <= duration.value)
}

fn sidechain_valid(
    value: &ResolvedSidechain,
    sequence: &ResolvedSequence,
    target: &ResolvedTrack,
    duration: RationalTime,
    timebase: u32,
) -> bool {
    finite(value.threshold_db, -60.0, 0.0)
        && finite(value.ratio, 1.0, 20.0)
        && finite(value.attack_ms, 0.01, 2000.0)
        && finite(value.release_ms, 0.01, 9000.0)
        && value
            .active_range
            .is_none_or(|range| local_range(range, duration, timebase))
        && source_valid(&value.source, sequence, target)
}

fn source_valid(
    source: &SidechainSource,
    sequence: &ResolvedSequence,
    target: &ResolvedTrack,
) -> bool {
    let matches_source = |track: &&ResolvedTrack| {
        track.id != target.id
            && track.state.audio_enabled
            && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
            && track
                .clips
                .iter()
                .any(|clip| clip.audio.as_ref().is_some_and(|audio| !audio.muted))
    };
    match source {
        SidechainSource::Track { track_id } => {
            sequence
                .tracks
                .iter()
                .filter(|track| track.id == *track_id)
                .filter(matches_source)
                .count()
                == 1
        }
        SidechainSource::Bus { bus_id } => {
            !route_is(target, bus_id.as_str())
                && sequence
                    .tracks
                    .iter()
                    .filter(|track| route_is(track, bus_id.as_str()))
                    .filter(matches_source)
                    .count()
                    > 0
        }
    }
}

fn route_is(track: &ResolvedTrack, expected: &str) -> bool {
    matches!(&track.routing.audio, Some(veac_plan::ResolvedAudioRoute::Bus { bus_id }) if bus_id == expected)
}

fn local_range(range: TimeRange, duration: RationalTime, timebase: u32) -> bool {
    range.start.is_valid()
        && range.duration.is_valid()
        && range.start.value >= 0
        && range.duration.value > 0
        && range.start.timescale == timebase
        && range.duration.timescale == timebase
        && duration.timescale == timebase
        && range.end().is_ok_and(|end| end <= duration)
}

fn finite(value: f64, minimum: f64, maximum: f64) -> bool {
    value.is_finite() && (minimum..=maximum).contains(&value)
}

fn invalid(check: &mut Check, clip: &ResolvedClip, message: &'static str) {
    check.push("PLAN_AUDIO_INVALID", Some(clip.id.to_string()), message);
}
