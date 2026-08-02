use veac_artifact::{
    BindingProvenanceKind, BoundResource, BoundStream, InputBinding, MediaRole, SourceClock,
};
use veac_plan::canonical::StreamSelection;
use veac_plan::ResolvedInput;

use crate::emitter::error::{diagnostic, CodegenErrorKind};
use crate::emitter::CodegenErrors;

pub(super) fn source(input: &ResolvedInput, binding: &InputBinding) -> Result<(), CodegenErrors> {
    if binding.source_identity() != Some(&input.observed_identity) {
        return Err(invalid(
            input,
            "binding logical source identity differs from the resolved input",
        ));
    }
    Ok(())
}

pub(super) fn stream(
    input: &ResolvedInput,
    binding: &InputBinding,
    role: MediaRole,
    stream: &BoundStream,
) -> Result<(), CodegenErrors> {
    let expected = selected(input, role);
    if expected.is_none() || binding.source_stream(role) != expected {
        return Err(invalid(
            input,
            "binding logical stream differs from the resolved stream selection",
        ));
    }
    if stream.resource().provenance_kind() == BindingProvenanceKind::Original {
        original_stream(input, stream, expected.expect("checked above"))?;
    }
    Ok(())
}

pub(super) fn resource(
    input: &ResolvedInput,
    resource: &BoundResource,
) -> Result<(), CodegenErrors> {
    if resource.identity() != &input.observed_identity {
        return Err(invalid(
            input,
            "bound resource identity differs from the resolved input",
        ));
    }
    Ok(())
}

fn original_stream(
    input: &ResolvedInput,
    stream: &BoundStream,
    expected: StreamSelection,
) -> Result<(), CodegenErrors> {
    let timescale = input
        .probe
        .as_ref()
        .and_then(|probe| probe.container_duration)
        .map_or(1, |time| time.timescale);
    let identity_clock = SourceClock::identity(timescale).ok();
    if stream.resource().identity() != &input.observed_identity
        || stream.physical_stream() != expected
        || Some(stream.clock()) != identity_clock
    {
        return Err(invalid(
            input,
            "original binding must preserve resource identity, stream, and source clock",
        ));
    }
    Ok(())
}

fn selected(input: &ResolvedInput, role: MediaRole) -> Option<StreamSelection> {
    match role {
        MediaRole::Video => input.video.as_ref().map(|value| value.selection),
        MediaRole::Audio => input.audio.as_ref().map(|value| value.selection),
    }
}

fn invalid(input: &ResolvedInput, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        "RESOURCE_BINDING_INVALID",
        Some(input.id.to_string()),
        message,
    ))
}
