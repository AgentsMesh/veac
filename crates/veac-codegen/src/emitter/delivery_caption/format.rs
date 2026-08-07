use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{CaptionSidecarFormat, RationalTime};

use super::{ass, failure::Failure, plain, Cue};

pub(super) fn render(
    format: CaptionSidecarFormat,
    cues: &[Cue<'_>],
    width: u32,
    height: u32,
    bindings: &ExecutionBindings,
) -> Result<String, Failure> {
    for cue in cues {
        super::semantics::validate(format, cue)?;
    }
    match format {
        CaptionSidecarFormat::Srt | CaptionSidecarFormat::WebVtt => plain::render(format, cues),
        CaptionSidecarFormat::Ass => ass::render(cues, width, height, bindings),
    }
}

pub(super) fn time_exact(format: CaptionSidecarFormat, value: RationalTime) -> bool {
    let units = if format == CaptionSidecarFormat::Ass {
        100
    } else {
        1_000
    };
    value.value >= 0
        && value.timescale > 0
        && i128::from(value.value) * i128::from(units) % i128::from(value.timescale) == 0
}

pub(super) fn timestamp(value: RationalTime, separator: char) -> String {
    let millis = i128::from(value.value) * 1_000 / i128::from(value.timescale);
    let hours = millis / 3_600_000;
    let minutes = millis / 60_000 % 60;
    let seconds = millis / 1_000 % 60;
    let fraction = millis % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}{separator}{fraction:03}")
}

pub(super) fn ass_timestamp(value: RationalTime) -> String {
    let value = i128::from(value.value) * 100 / i128::from(value.timescale);
    let hours = value / 360_000;
    let minutes = value / 6_000 % 60;
    let seconds = value / 100 % 60;
    let centiseconds = value % 100;
    format!("{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}")
}
