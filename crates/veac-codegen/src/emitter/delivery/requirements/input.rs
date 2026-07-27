use std::path::Path;

use veac_artifact::{ArtifactKind, BoundResource, ExecutionBindings, MediaRole};
use veac_plan::{ResolvedInput, ResolvedRenderPlan};

use super::super::super::{
    error::diagnostic, BackendCapabilityKind, BackendCommand, CodegenErrorKind, CodegenErrors,
};

pub(super) fn capabilities(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    command: &BackendCommand,
) -> Result<Vec<(BackendCapabilityKind, String)>, CodegenErrors> {
    let mut values = Vec::new();
    for (input_index, backend_input) in command.inputs.iter().enumerate() {
        let mut matched = false;
        for input in &plan.inputs {
            let Some(binding) = bindings.input(&input.id) else {
                continue;
            };
            for role in [MediaRole::Video, MediaRole::Audio] {
                let Some(stream) = binding.stream(role) else {
                    continue;
                };
                if stream.resource().path() == backend_input.path {
                    matched = true;
                    if uses_stream(command, input_index, stream.physical_stream().global_index) {
                        append(&mut values, input, stream.resource(), role)?;
                    }
                }
            }
        }
        if bindings
            .full_render_segment()
            .is_some_and(|segment| segment.resource().path() == backend_input.path)
        {
            matched = true;
            insert(&mut values, BackendCapabilityKind::Demuxer, "mov");
        }
        if !matched {
            return Err(invalid(&backend_input.path));
        }
    }
    Ok(values)
}

fn uses_stream(command: &BackendCommand, input_index: usize, global_index: u32) -> bool {
    let direct = format!("{input_index}:{global_index}");
    command
        .filter_graph
        .as_ref()
        .is_some_and(|graph| graph.contains(&format!("[{direct}]")))
        || command.maps.iter().any(|value| value == &direct)
}

fn append(
    values: &mut Vec<(BackendCapabilityKind, String)>,
    input: &ResolvedInput,
    resource: &BoundResource,
    role: MediaRole,
) -> Result<(), CodegenErrors> {
    let (decoder, demuxer) = match resource.artifact_kind() {
        Some(ArtifactKind::ProxyVideo) if role == MediaRole::Video => ("h264", "mov"),
        Some(ArtifactKind::ProxyAudio) if role == MediaRole::Audio => ("pcm_s16le", "wav"),
        Some(_) => return Err(invalid(resource.path())),
        None => (source_decoder(input, role)?, source_demuxer(input)?),
    };
    insert(values, BackendCapabilityKind::Decoder, decoder);
    insert(values, BackendCapabilityKind::Demuxer, demuxer);
    Ok(())
}

fn source_decoder(input: &ResolvedInput, role: MediaRole) -> Result<&str, CodegenErrors> {
    match role {
        MediaRole::Video => input.video.as_ref().map(|value| value.codec.as_str()),
        MediaRole::Audio => input.audio.as_ref().map(|value| value.codec.as_str()),
    }
    .ok_or_else(|| invalid(Path::new(&input.canonical_uri)))
}

fn source_demuxer(input: &ResolvedInput) -> Result<&str, CodegenErrors> {
    input
        .probe
        .as_ref()
        .and_then(|probe| probe.container_format.split(',').next())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid(Path::new(&input.canonical_uri)))
}

fn insert(
    values: &mut Vec<(BackendCapabilityKind, String)>,
    kind: BackendCapabilityKind,
    name: &str,
) {
    if !values
        .iter()
        .any(|value| value.0 == kind && value.1 == name)
    {
        values.push((kind, name.to_owned()));
    }
}

fn invalid(path: &Path) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        "BACKEND_INPUT_CAPABILITY_UNRESOLVED",
        None,
        format!(
            "cannot resolve decoder and demuxer requirements for {}",
            path.display()
        ),
    ))
}

#[cfg(test)]
#[path = "input/tests.rs"]
mod tests;
