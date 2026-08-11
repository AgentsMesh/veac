use std::ffi::OsString;
use std::path::Path;
use std::time::Instant;

use veac_ir::{MediaProbeSnapshot, Rational, VideoCadence};

use super::process;
use crate::input_policy;

mod evidence;
pub(super) use evidence::Evidence;

const MAX_CADENCE_OUTPUT_BYTES: u64 = veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES;

pub(super) enum Inspection {
    Skipped,
    Attempted(Option<Evidence>),
}

pub(super) fn inspect(
    snapshot: &MediaProbeSnapshot,
    binary: &Path,
    source: &Path,
    deadline: Instant,
) -> Inspection {
    inspect_with_limit(snapshot, binary, source, deadline, MAX_CADENCE_OUTPUT_BYTES)
}

fn inspect_with_limit(
    snapshot: &MediaProbeSnapshot,
    binary: &Path,
    source: &Path,
    deadline: Instant,
    output_limit: u64,
) -> Inspection {
    let Some(selection) = snapshot.selected_video_stream else {
        return Inspection::Skipped;
    };
    let Some(time_base) = snapshot
        .streams
        .iter()
        .find(|stream| stream.global_index == selection.global_index)
        .and_then(|stream| stream.time_base)
    else {
        return Inspection::Skipped;
    };
    let Some(frame_rate) = snapshot
        .streams
        .iter()
        .find(|stream| stream.global_index == selection.global_index)
        .and_then(|stream| stream.video.as_ref())
        .map(|video| video.frame_rate)
    else {
        return Inspection::Skipped;
    };
    let output = process::run(
        binary,
        &arguments(source, selection.type_index),
        output_limit,
        deadline,
        "video cadence inspection",
    );
    let evidence = output.ok().and_then(|output| {
        output
            .status
            .success()
            .then(|| evidence::classify(&output.stdout, time_base, frame_rate))
    });
    Inspection::Attempted(evidence)
}

pub(super) fn apply(snapshot: &mut MediaProbeSnapshot, evidence: Evidence) {
    let Some(selection) = snapshot.selected_video_stream else {
        return;
    };
    if let Some(video) = snapshot
        .streams
        .iter_mut()
        .find(|stream| stream.global_index == selection.global_index)
        .and_then(|stream| stream.video.as_mut())
    {
        video.cadence = evidence.cadence;
        if let Some(rate) = evidence.frame_rate {
            video.frame_rate = Some(rate);
        }
    }
}

fn arguments(source: &Path, type_index: u32) -> Vec<OsString> {
    let selection = format!("v:{type_index}");
    let mut values = [
        "-v",
        "error",
        "-select_streams",
        selection.as_str(),
        "-show_packets",
        "-show_entries",
        "packet=pts,duration",
        "-of",
        "json",
    ]
    .into_iter()
    .map(OsString::from)
    .collect::<Vec<_>>();
    values.extend(input_policy::os_arguments());
    values.push(OsString::from("-i"));
    values.push(source.as_os_str().to_owned());
    values
}

pub(super) fn usable(evidence: Evidence) -> bool {
    evidence.cadence != VideoCadence::Unknown
}

pub(super) fn rate_within_limit(rate: Rational) -> bool {
    i128::from(rate.numerator) <= i128::from(veac_ir::MAX_FRAME_RATE) * i128::from(rate.denominator)
}

#[cfg(test)]
#[path = "cadence/tests.rs"]
mod tests;
