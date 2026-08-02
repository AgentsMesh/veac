use crate::error::{CliError, CliResult};

pub(super) fn runtime<T>(value: Result<T, veac_runtime::RuntimeError>, code: &str) -> CliResult<T> {
    value.map_err(|error| {
        if error.kind == veac_runtime::RuntimeErrorKind::ResourceLimit {
            CliError::resource_limit(code, error.to_string())
        } else {
            CliError::new(code, error.to_string())
        }
    })
}

pub(super) fn probe<T>(
    value: Result<T, veac_runtime::asset::ProbeError>,
    code: &str,
) -> CliResult<T> {
    value.map_err(|error| {
        if matches!(error, veac_runtime::asset::ProbeError::ResourceLimit { .. }) {
            CliError::resource_limit(code, error.to_string())
        } else {
            CliError::new(code, error.to_string())
        }
    })
}

pub(super) fn workflow<T>(
    value: veac_runtime::workflow::WorkflowResult<T>,
    code: &str,
) -> CliResult<T> {
    value.map_err(|error| {
        if error.kind == veac_runtime::workflow::WorkflowErrorKind::ResourceLimit {
            CliError::resource_limit(code, error.to_string())
        } else {
            CliError::new(code, error.to_string())
        }
    })
}
