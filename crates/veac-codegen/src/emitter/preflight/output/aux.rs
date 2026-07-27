use veac_plan::canonical::{
    AudioCodec, AudioStemFormat, AudioStemSource, CaptionSidecarFormat, Deliverable,
    DeliverableKind, ImageFormat, RationalTime, TrackKind,
};
use veac_plan::{ResolvedRenderPlan, ResolvedSequence};

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan, value: &Deliverable) {
    let Some(sequence) = plan
        .sequences
        .iter()
        .find(|item| item.id == plan.entry_sequence_id)
    else {
        return;
    };
    let (valid, code, message) = match &value.kind {
        DeliverableKind::ImageSequence(settings) => (
            image_extension(&value.file_name, settings.format)
                && veac_plan::canonical::image_sequence_range_valid(
                    settings.start_number,
                    sequence.duration,
                    plan.output.frame_rate,
                ),
            "PLAN_IMAGE_SEQUENCE_INVALID",
            "image sequence extension is incompatible with its format",
        ),
        DeliverableKind::CaptionSidecar(settings) => (
            caption(value, settings, sequence),
            "PLAN_CAPTION_SIDECAR_INVALID",
            "caption sidecar format, tracks, or cue timing is invalid",
        ),
        DeliverableKind::AudioStem(settings) => (
            stem(value, settings, sequence),
            "PLAN_AUDIO_STEM_INVALID",
            "audio stem format, codec, or source is invalid",
        ),
        DeliverableKind::Scope(settings) => (
            image_extension(&value.file_name, settings.format)
                && veac_plan::canonical::ffmpeg_dimensions_valid(settings.width, settings.height)
                && local_snapshot(
                    settings.at,
                    sequence.duration,
                    plan.header.source.timebase,
                    plan.output.frame_rate,
                ),
            "PLAN_SCOPE_OUTPUT_INVALID",
            "scope format, time, or default untrusted-plan render budget is invalid",
        ),
        DeliverableKind::Video(_) => (
            true,
            "PLAN_AUX_OUTPUT_INVALID",
            "auxiliary output is invalid",
        ),
    };
    if !valid {
        check.push(code, Some(value.id.to_string()), message);
    }
}

fn caption(
    value: &Deliverable,
    settings: &veac_plan::canonical::CaptionSidecarOutput,
    sequence: &ResolvedSequence,
) -> bool {
    let expected = match settings.format {
        CaptionSidecarFormat::Srt => "srt",
        CaptionSidecarFormat::WebVtt => "vtt",
        CaptionSidecarFormat::Ass => "ass",
    };
    extension_is(&value.file_name, expected)
        && !settings.track_ids.is_empty()
        && settings.track_ids.windows(2).all(|pair| pair[0] < pair[1])
        && settings.track_ids.iter().all(|id| {
            let mut matches = sequence
                .tracks
                .iter()
                .filter(|track| track.id == *id && track.kind == TrackKind::Caption);
            let valid = matches.next().is_some() && matches.next().is_none();
            valid
                && sequence
                    .tracks
                    .iter()
                    .find(|track| track.id == *id)
                    .is_some_and(|track| {
                        track
                            .clips
                            .iter()
                            .all(|clip| caption_exact(settings.format, clip.record_range))
                    })
        })
}

fn stem(
    value: &Deliverable,
    settings: &veac_plan::canonical::AudioStemOutput,
    sequence: &ResolvedSequence,
) -> bool {
    let extension = match settings.format {
        AudioStemFormat::Wav => "wav",
        AudioStemFormat::Flac => "flac",
    };
    extension_is(&value.file_name, extension)
        && veac_plan::canonical::audio_output_valid(&settings.audio)
        && matches!(
            (settings.format, settings.audio.codec),
            (AudioStemFormat::Wav, AudioCodec::PcmS16Le)
                | (AudioStemFormat::Wav, AudioCodec::PcmS24Le)
                | (AudioStemFormat::Wav, AudioCodec::PcmS32Le)
                | (AudioStemFormat::Flac, AudioCodec::Flac)
        )
        && match &settings.source {
            AudioStemSource::Master => true,
            AudioStemSource::Track { track_id } => sequence.tracks.iter().any(|track| {
                track.id == *track_id && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
            }),
            AudioStemSource::Bus { bus_id } => {
                sequence.tracks.iter().any(|track| {
                    matches!(&track.routing.audio, Some(veac_plan::ResolvedAudioRoute::Bus { bus_id: value }) if value == bus_id.as_str())
                })
            }
        }
}

fn local_snapshot(
    value: RationalTime,
    duration: RationalTime,
    timebase: u32,
    frame_rate: veac_plan::canonical::Rational,
) -> bool {
    value.is_valid()
        && duration.is_valid()
        && value.timescale == timebase
        && duration.timescale == timebase
        && value.value >= 0
        && value < duration
        && crate::emitter::time::containing_frame(value, frame_rate).is_some()
}

fn caption_exact(format: CaptionSidecarFormat, range: veac_plan::canonical::TimeRange) -> bool {
    let units = if format == CaptionSidecarFormat::Ass {
        100
    } else {
        1_000
    };
    exact_units(range.start, units) && range.end().is_ok_and(|value| exact_units(value, units))
}

fn exact_units(value: RationalTime, units: i128) -> bool {
    value.value >= 0
        && value.timescale > 0
        && i128::from(value.value) * units % i128::from(value.timescale) == 0
}

fn image_extension(value: &str, format: ImageFormat) -> bool {
    extension_is(
        value,
        match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Exr => "exr",
        },
    )
}

fn extension_is(value: &str, expected: &str) -> bool {
    value
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case(expected))
}
