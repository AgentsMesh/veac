use std::collections::BTreeSet;
use std::path::PathBuf;

use veac_codegen::emitter::{BackendAction, BackendFilterBinding, MAX_FILTER_GRAPH_BYTES};

use super::invalid;
use crate::executor::model::RuntimeBundle;
use crate::RuntimeError;

pub(super) fn validate(bundle: &RuntimeBundle) -> Result<(), RuntimeError> {
    let protected = bundle
        .protected_resources
        .iter()
        .map(|resource| resource.path.clone())
        .collect::<BTreeSet<_>>();
    for command in bundle.tasks.iter().filter_map(|task| match &task.action {
        BackendAction::Ffmpeg(command) => Some(command),
        BackendAction::WriteFile { .. } => None,
    }) {
        validate_command(command, &protected)?;
    }
    Ok(())
}

fn validate_command(
    command: &veac_codegen::emitter::BackendCommand,
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    match (&command.filter_graph, &command.filter_contract) {
        (None, None) => return Ok(()),
        (Some(graph), None) if !graph.contains("__VEAC_FILTER_RESOURCE_") => {
            return validate_size(graph)
        }
        (Some(_), None) => {
            return invalid("FFmpeg filter graph contains an unbound resource token")
        }
        (None, Some(_)) => return invalid("FFmpeg filter contract has no rendered filter graph"),
        (Some(graph), Some(contract)) => {
            let original = contract.render_original().map_err(|error| {
                RuntimeError::new(format!("invalid backend filter contract: {error}"))
            })?;
            if original != *graph {
                return invalid("FFmpeg filter graph does not match its typed resource contract");
            }
            validate_size(graph)?;
            for binding in contract.bindings() {
                validate_binding(binding, protected)?;
            }
        }
    }
    Ok(())
}

fn validate_binding(
    binding: &BackendFilterBinding,
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    let mut unique = BTreeSet::new();
    for path in binding.files() {
        if !unique.insert(path) {
            return invalid("filter resource binding contains a duplicate file");
        }
        if !protected.contains(path) {
            return invalid("every filter file must be a protected bundle resource");
        }
    }
    Ok(())
}

pub(in crate::executor) fn validate_size(graph: &str) -> Result<(), RuntimeError> {
    if graph.len() > MAX_FILTER_GRAPH_BYTES {
        invalid("FFmpeg filter graph exceeds the backend graph limit")
    } else {
        Ok(())
    }
}
