use crate::*;

use super::super::super::Validator;

pub(super) fn validate(
    validator: &mut Validator,
    value: &Deliverable,
    settings: &CaptionSidecarOutput,
    project: &Project,
    sequence_id: &SequenceId,
    path: &str,
) {
    let expected = match settings.format {
        CaptionSidecarFormat::Srt => "srt",
        CaptionSidecarFormat::WebVtt => "vtt",
        CaptionSidecarFormat::Ass => "ass",
    };
    if !extension_is(&value.file_name, expected)
        || settings.track_ids.is_empty()
        || !settings.track_ids.windows(2).all(|pair| pair[0] < pair[1])
    {
        validator.value_error("OUTPUT_CAPTION_SIDECAR", path, value.id.as_str());
    }
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == *sequence_id);
    for id in &settings.track_ids {
        let track = sequence
            .into_iter()
            .flat_map(|value| &value.tracks)
            .find(|track| track.id == *id && track.kind == TrackKind::Caption);
        let Some(track) = track else {
            validator.missing_ref("OUTPUT_CAPTION_TRACK_NOT_FOUND", id.as_str(), path);
            continue;
        };
        if track
            .clips
            .iter()
            .any(|clip| !range_exact(settings.format, clip.record_range))
        {
            validator.value_error("OUTPUT_CAPTION_TIME", path, value.id.as_str());
        }
    }
}

fn range_exact(format: CaptionSidecarFormat, range: TimeRange) -> bool {
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

fn extension_is(value: &str, expected: &str) -> bool {
    value
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case(expected))
}
