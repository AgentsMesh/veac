use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendFilterBinding, BackendInternalAccess,
    MAX_FILTER_GRAPH_BYTES,
};

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
    command: &BackendCommand,
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    let mut produced = BTreeSet::new();
    let mut declared = BTreeSet::new();
    for preparation in &command.preparations {
        if !preparation.command.preparations.is_empty() {
            return invalid("FFmpeg preparation may not contain nested preparations");
        }
        if preparation.outputs.is_empty() {
            return invalid("FFmpeg preparation must declare at least one sidecar output");
        }
        for path in &preparation.outputs {
            safe_internal_path(path)?;
            if !declared.insert(path.clone()) {
                return invalid("FFmpeg preparation sidecar outputs must be unique");
            }
        }
        validate_inputs(&preparation.command, protected)?;
        validate_graph(
            &preparation.command,
            protected,
            &produced,
            Some(&preparation.outputs),
        )?;
        let producers = internal_produces(&preparation.command);
        if producers.len() != preparation.outputs.len()
            || preparation
                .outputs
                .iter()
                .any(|path| producers.get(path) != Some(&1))
        {
            return invalid(
                "FFmpeg preparation outputs must each have exactly one internal producer",
            );
        }
        produced.extend(preparation.outputs.iter().cloned());
    }
    validate_inputs(command, protected)?;
    validate_graph(command, protected, &produced, None)
}

fn internal_produces(command: &BackendCommand) -> BTreeMap<PathBuf, usize> {
    command
        .filter_contract
        .iter()
        .flat_map(|contract| contract.bindings())
        .filter_map(|binding| match binding {
            BackendFilterBinding::InternalFile {
                path,
                access: BackendInternalAccess::Produce,
                ..
            } => Some(path.clone()),
            _ => None,
        })
        .fold(BTreeMap::new(), |mut produced, path| {
            *produced.entry(path).or_default() += 1;
            produced
        })
}

fn validate_inputs(
    command: &BackendCommand,
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    if command
        .inputs
        .iter()
        .any(|input| !protected.contains(&input.path))
    {
        return invalid("every FFmpeg input must be a protected bundle resource");
    }
    Ok(())
}

fn validate_graph(
    command: &BackendCommand,
    protected: &BTreeSet<PathBuf>,
    produced: &BTreeSet<PathBuf>,
    preparation_outputs: Option<&[PathBuf]>,
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
                validate_binding(binding, protected, produced, preparation_outputs)?;
            }
        }
    }
    Ok(())
}

fn validate_binding(
    binding: &BackendFilterBinding,
    protected: &BTreeSet<PathBuf>,
    produced: &BTreeSet<PathBuf>,
    preparation_outputs: Option<&[PathBuf]>,
) -> Result<(), RuntimeError> {
    if let BackendFilterBinding::InternalFile { path, access, .. } = binding {
        safe_internal_path(path)?;
        return match (preparation_outputs, access) {
            (Some(outputs), BackendInternalAccess::Produce) if outputs.contains(path) => Ok(()),
            (Some(_), BackendInternalAccess::Produce) => {
                invalid("FFmpeg preparation may only produce its declared sidecar outputs")
            }
            (None, BackendInternalAccess::Produce) => {
                invalid("main FFmpeg command may only consume internal sidecars")
            }
            (_, BackendInternalAccess::Consume) if produced.contains(path) => Ok(()),
            _ => invalid("FFmpeg internal sidecar is consumed before it is produced"),
        };
    }
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

fn safe_internal_path(path: &Path) -> Result<(), RuntimeError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|value| !matches!(value, Component::Normal(_)))
    {
        invalid("FFmpeg internal sidecar path must be a non-empty relative path without traversal")
    } else {
        Ok(())
    }
}

pub(in crate::executor) fn validate_size(graph: &str) -> Result<(), RuntimeError> {
    if graph.len() > MAX_FILTER_GRAPH_BYTES {
        invalid("FFmpeg filter graph exceeds the backend graph limit")
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "filter/tests.rs"]
mod tests;
