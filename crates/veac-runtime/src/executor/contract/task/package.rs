use std::path::{Component, Path};

use veac_codegen::emitter::{BackendCommand, BackendPackagePaths};

use crate::executor::contract::invalid;
use crate::executor::output;
use crate::RuntimeError;

pub(super) fn validate(
    command: &BackendCommand,
    entrypoint: &Path,
    paths: &BackendPackagePaths,
) -> Result<(), RuntimeError> {
    leaf(entrypoint)?;
    leaf(&paths.playlist_pattern)?;
    leaf(&paths.segment_pattern)?;
    if command.output_path != paths.playlist_pattern {
        return invalid("HLS command output must equal its playlist pattern");
    }
    exact_option(
        &command.output_args,
        "-master_pl_name",
        &output::path_string(entrypoint),
    )?;
    exact_option(
        &command.output_args,
        "-hls_segment_filename",
        &output::path_string(&paths.segment_pattern),
    )?;
    for (name, value) in [
        ("-f", "hls"),
        ("-hls_playlist_type", "vod"),
        ("-hls_list_size", "0"),
    ] {
        exact_option(&command.output_args, name, value)?;
    }
    Ok(())
}

fn leaf(path: &Path) -> Result<(), RuntimeError> {
    let mut components = path.components();
    if matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none() {
        Ok(())
    } else {
        invalid("HLS package paths must be safe relative leaf patterns")
    }
}

fn exact_option(arguments: &[String], name: &str, value: &str) -> Result<(), RuntimeError> {
    let matching = arguments
        .windows(2)
        .filter(|pair| pair[0] == name && pair[1] == value)
        .count();
    let named = arguments
        .iter()
        .filter(|argument| *argument == name)
        .count();
    if matching == 1 && named == 1 {
        Ok(())
    } else {
        invalid("HLS command does not match its typed package layout")
    }
}
