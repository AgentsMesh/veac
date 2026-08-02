use std::ffi::OsString;
use std::path::Path;

use veac_artifact::{MediaArtifactLimits, MediaArtifactSpec, SourceClockSpec};
use veac_ir::{Rational, RationalTime};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::input_policy;

mod spec;

pub(super) fn arguments(
    input: &Path,
    output: &Path,
    artifact: &MediaArtifactSpec,
    limits: MediaArtifactLimits,
) -> WorkflowResult<Vec<OsString>> {
    let mut args = strings(&["-nostdin", "-hide_banner", "-loglevel", "error", "-y"]);
    args.extend(strings(&["-timelimit"]));
    push(&mut args, limits.max_derivation_cpu_seconds.to_string());
    args.extend(input_policy::os_arguments());
    args.push(OsString::from("-i"));
    args.push(input.as_os_str().to_owned());
    spec::append(&mut args, artifact)?;
    args.extend(strings(&[
        "-map_metadata",
        "-1",
        "-map_chapters",
        "-1",
        "-fflags",
        "+bitexact",
        "-fs",
    ]));
    push(&mut args, limits.max_payload_bytes.to_string());
    args.push(output.as_os_str().to_owned());
    Ok(args)
}

pub(super) fn video_encoding(crf: u8) -> Vec<OsString> {
    let mut value = strings(&["-c:v", "libx264", "-preset", "medium", "-crf"]);
    push(&mut value, crf.to_string());
    value.extend(strings(&[
        "-pix_fmt",
        "yuv420p",
        "-threads",
        "1",
        "-flags:v",
        "+bitexact",
        "-movflags",
        "+faststart",
    ]));
    value
}

pub(super) fn source_clock(args: &mut Vec<OsString>, value: SourceClockSpec) -> WorkflowResult<()> {
    match value {
        SourceClockSpec::Identity { duration } => {
            args.extend(strings(&["-t"]));
            push(args, seconds(duration)?);
            Ok(())
        }
        SourceClockSpec::Bounded { logical_range } => {
            args.extend(strings(&["-ss"]));
            push(args, seconds(logical_range.start)?);
            args.extend(strings(&["-t"]));
            push(args, seconds(logical_range.duration)?);
            Ok(())
        }
    }
}

pub(super) fn scale(width: u32, height: u32) -> String {
    format!(
        "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"
    )
}

pub(super) fn ratio(value: Rational) -> String {
    format!("{}/{}", value.numerator, value.denominator)
}

pub(super) fn seconds(value: RationalTime) -> WorkflowResult<String> {
    let Some(scaled) = i128::from(value.value).checked_mul(1_000_000) else {
        return invalid_time();
    };
    if !value.is_valid() || value.value < 0 || scaled % i128::from(value.timescale) != 0 {
        return invalid_time();
    }
    let micros = scaled / i128::from(value.timescale);
    let whole = micros / 1_000_000;
    let fraction = micros % 1_000_000;
    if fraction == 0 {
        Ok(whole.to_string())
    } else {
        Ok(format!("{whole}.{fraction:06}")
            .trim_end_matches('0')
            .to_owned())
    }
}

fn invalid_time<T>() -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::InvalidContract,
        "artifact time is not exactly representable by FFmpeg",
    ))
}

pub(super) fn strings(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

pub(super) fn push(values: &mut Vec<OsString>, value: String) {
    values.push(OsString::from(value));
}

#[cfg(test)]
#[path = "command/optical_tests.rs"]
mod optical_tests;
#[cfg(test)]
#[path = "command/tests.rs"]
mod tests;
