use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use veac_codegen::emitter::BackendRequirement;

use super::FfmpegEnvironment;
use crate::{RuntimeError, RuntimeErrorKind};

pub(super) fn validate(
    environment: &dyn FfmpegEnvironment,
    requirements: &[BackendRequirement],
    deadline: Instant,
) -> Result<(), RuntimeError> {
    let kinds = requirements
        .iter()
        .map(BackendRequirement::kind)
        .collect::<BTreeSet<_>>();
    let mut available = BTreeMap::new();
    for kind in kinds {
        crate::executor::deadline::ensure_setup(deadline)?;
        available.insert(kind, environment.capability_until(kind, deadline)?);
    }
    let missing = requirements
        .iter()
        .filter(|value| !available[&value.kind()].contains(value.name()))
        .map(|value| {
            format!(
                "{}: {} {}",
                value.deliverable_id(),
                value.kind().as_str(),
                value.name()
            )
        })
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        let error = RuntimeError::missing_backend_capability(format!(
            "required FFmpeg capabilities are unavailable: {}",
            missing.join(", ")
        ));
        debug_assert_eq!(error.kind, RuntimeErrorKind::MissingBackendCapability);
        Err(error)
    }
}

pub(super) fn parse_encoders(output: &str) -> BTreeSet<String> {
    parse_codec_table(output)
}

pub(super) fn parse_decoders(output: &str) -> BTreeSet<String> {
    parse_codec_table(output)
}

fn parse_codec_table(output: &str) -> BTreeSet<String> {
    parse_table(output, |flags| {
        flags.len() == 6
            && flags
                .bytes()
                .all(|byte| byte == b'.' || byte.is_ascii_uppercase())
    })
}

pub(super) fn parse_muxers(output: &str) -> BTreeSet<String> {
    parse_table(output, |flags| flags == "E")
}

pub(super) fn parse_demuxers(output: &str) -> BTreeSet<String> {
    parse_table(output, |flags| flags == "D")
}

pub(super) fn parse_filters(output: &str) -> BTreeSet<String> {
    parse_table(output, |flags| {
        flags.len() == 2 && flags.bytes().all(|byte| matches!(byte, b'.' | b'T' | b'S'))
    })
}

pub(super) fn parse_names(output: &str) -> BTreeSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            (fields.next().is_none() && valid_name(name)).then(|| name.to_owned())
        })
        .collect()
}

fn parse_table(output: &str, valid_flags: impl Fn(&str) -> bool) -> BTreeSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let flags = fields.next()?;
            let names = fields.next()?;
            valid_flags(flags).then_some(names)
        })
        .flat_map(|names| names.split(','))
        .filter(|name| valid_name(name))
        .map(str::to_owned)
        .collect()
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
