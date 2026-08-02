use veac_artifact::{BindingProvenanceKind, BoundStream, MediaRole, SourceClock};
use veac_plan::canonical::{MaterialKind, RationalTime};
use veac_plan::{ResolvedInput, ResolvedInputKind};

use super::errors::invalid;
use super::CodegenErrors;

pub(super) fn clock(
    input: &ResolvedInput,
    role: MediaRole,
    stream: &BoundStream,
) -> Result<SourceClock, CodegenErrors> {
    if stream.resource().provenance_kind() != BindingProvenanceKind::Original
        || time_invariant(input)
    {
        return Ok(stream.clock());
    }
    let Some(duration) = duration(input, role) else {
        return Ok(stream.clock());
    };
    SourceClock::original_stream(duration, start_time(input, role))
        .map_err(|error| invalid(&format!("invalid original stream clock: {error}")))
}

pub(super) fn duration(input: &ResolvedInput, role: MediaRole) -> Option<RationalTime> {
    let probe = input.probe.as_ref()?;
    let (duration, start) = match role {
        MediaRole::Video => {
            let stream = input.video.as_ref()?;
            (stream.duration, stream.start_time)
        }
        MediaRole::Audio => {
            let stream = input.audio.as_ref()?;
            (stream.duration, stream.start_time)
        }
    };
    duration.or_else(|| {
        start
            .map(|value| value.value == 0)
            .unwrap_or(true)
            .then_some(probe.container_duration)
            .flatten()
    })
}

fn start_time(input: &ResolvedInput, role: MediaRole) -> Option<RationalTime> {
    match role {
        MediaRole::Video => input.video.as_ref()?.start_time,
        MediaRole::Audio => input.audio.as_ref()?.start_time,
    }
}

pub(super) fn time_invariant(input: &ResolvedInput) -> bool {
    matches!(
        input.kind,
        ResolvedInputKind::Media {
            material_kind: MaterialKind::Image
        } | ResolvedInputKind::Resource {
            material_kind: MaterialKind::Image
        }
    )
}
