use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{BackendFilterBinding, BackendFilterContract, BackendFilterEscape};

pub(super) fn original(contract: &BackendFilterContract) -> Result<String, String> {
    render(contract, |binding| match binding {
        BackendFilterBinding::InternalFile { token, .. } => Some(PathBuf::from(token)),
        _ => Some(binding.original().to_path_buf()),
    })
}

pub(super) fn bound(
    contract: &BackendFilterContract,
    files: &BTreeMap<PathBuf, PathBuf>,
    directories: &BTreeMap<Vec<PathBuf>, PathBuf>,
) -> Result<String, String> {
    render(contract, |binding| match binding {
        BackendFilterBinding::File { path, .. } => files.get(path).cloned(),
        BackendFilterBinding::Directory { files, .. } => directories.get(files).cloned(),
        BackendFilterBinding::InternalFile { token, .. } => Some(PathBuf::from(token)),
    })
}

pub(super) fn internal(
    contract: &BackendFilterContract,
    graph: &str,
    root: &Path,
) -> Result<String, String> {
    let mut graph = graph.to_owned();
    for binding in contract.bindings() {
        let BackendFilterBinding::InternalFile {
            token,
            path,
            access: _,
            escape: syntax,
        } = binding
        else {
            continue;
        };
        if graph.matches(token).count() != 1 {
            return Err("internal filter token must occur exactly once".to_owned());
        }
        let value = root.join(path);
        let text = value
            .to_str()
            .ok_or_else(|| "internal filter path is not valid UTF-8".to_owned())?;
        graph = graph.replacen(token, &escape(text, *syntax), 1);
    }
    Ok(graph)
}

fn render(
    contract: &BackendFilterContract,
    mut resolve: impl FnMut(&BackendFilterBinding) -> Option<PathBuf>,
) -> Result<String, String> {
    contract.validate_tokens()?;
    let mut replacements = Vec::with_capacity(contract.bindings.len());
    for binding in &contract.bindings {
        let path = resolve(binding)
            .ok_or_else(|| "filter resource has no verified path binding".to_owned())?;
        let text = path
            .to_str()
            .ok_or_else(|| "filter resource path is not valid UTF-8".to_owned())?;
        let start = contract
            .template
            .find(binding.token())
            .ok_or_else(|| "filter resource token is absent from template".to_owned())?;
        replacements.push((start, binding.token().len(), escape(text, binding.escape())));
    }
    replacements.sort_unstable_by(|left, right| right.0.cmp(&left.0));
    let mut graph = contract.template.clone();
    for (start, length, replacement) in replacements {
        graph.replace_range(start..start + length, &replacement);
    }
    Ok(graph)
}

fn escape(value: &str, syntax: BackendFilterEscape) -> String {
    match syntax {
        BackendFilterEscape::Quoted => quoted(value),
        BackendFilterEscape::FilterValue => filter_value(value),
    }
}

fn quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace(':', "\\:")
        .replace(',', "\\,")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace(';', "\\;")
}

fn filter_value(value: &str) -> String {
    let mut option = String::new();
    for character in value.chars() {
        if matches!(character, '\\' | '\'' | ':') {
            option.push('\\');
        }
        option.push(character);
    }
    let mut graph = String::new();
    for character in option.chars() {
        if matches!(character, '\\' | '\'' | '[' | ']' | ',' | ';') {
            graph.push('\\');
        }
        graph.push(character);
    }
    graph
}
