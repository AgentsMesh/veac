use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use veac_codegen::emitter::{BackendAction, BackendPhase};

use super::super::model::RuntimeBundle;
use super::invalid;
use crate::RuntimeError;

pub(super) fn validate(bundle: &RuntimeBundle) -> Result<(), RuntimeError> {
    let mut first_pass = BTreeMap::<String, PathBuf>::new();
    for task in &bundle.tasks {
        let BackendAction::Ffmpeg(command) = &task.action else {
            continue;
        };
        let pass = option(&command.output_args, "-pass")?;
        let prefix = option(&command.output_args, "-passlogfile")?;
        match task.phase {
            BackendPhase::Single if pass.is_none() && prefix.is_none() => {}
            BackendPhase::Single => {
                return invalid("single-pass task may not declare FFmpeg pass state")
            }
            BackendPhase::FirstPass if pass == Some("1") => {
                let prefix = required_prefix(prefix)?;
                first_pass.insert(task.deliverable_id.to_string(), prefix);
            }
            BackendPhase::SecondPass if pass == Some("2") => {
                let prefix = required_prefix(prefix)?;
                if first_pass.get(&task.deliverable_id.to_string()) != Some(&prefix) {
                    return invalid("two-pass tasks must use the same FFmpeg passlog prefix");
                }
            }
            BackendPhase::FirstPass | BackendPhase::SecondPass => {
                return invalid("two-pass task has an invalid FFmpeg pass number")
            }
        }
    }
    Ok(())
}

fn option<'a>(arguments: &'a [String], name: &str) -> Result<Option<&'a str>, RuntimeError> {
    let positions: Vec<_> = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (value == name).then_some(index))
        .collect();
    match positions.as_slice() {
        [] => Ok(None),
        [index] if *index + 1 < arguments.len() => Ok(Some(arguments[*index + 1].as_str())),
        _ => invalid("FFmpeg pass option must occur exactly once with one value"),
    }
}

fn required_prefix(value: Option<&str>) -> Result<PathBuf, RuntimeError> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return invalid("two-pass task requires one non-empty FFmpeg passlog prefix");
    };
    let path = Path::new(value);
    if path.file_name().is_none() {
        return invalid("FFmpeg passlog prefix must have a file name");
    }
    Ok(path.to_path_buf())
}
