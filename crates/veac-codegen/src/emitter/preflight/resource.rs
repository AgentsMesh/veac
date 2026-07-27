mod cube;

use std::collections::BTreeSet;

use veac_artifact::{ArtifactErrorKind, ExecutionBindings};
use veac_plan::canonical::MaterialKind;
use veac_plan::{PlanInputId, ResolvedInput, ResolvedInputKind, ResolvedRenderPlan};

use super::super::error::{diagnostic, CodegenErrorKind};
use super::super::CodegenErrors;

pub(super) fn validate(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    used: &BTreeSet<PlanInputId>,
) -> Result<(), CodegenErrors> {
    for input in &plan.inputs {
        if !used.contains(&input.id) {
            continue;
        }
        let ResolvedInputKind::Resource { material_kind } = input.kind else {
            continue;
        };
        if matches!(material_kind, MaterialKind::Lut1d | MaterialKind::Lut3d) {
            validate_lut(input, material_kind, bindings)?;
        }
    }
    Ok(())
}

fn validate_lut(
    input: &ResolvedInput,
    expected: MaterialKind,
    bindings: &ExecutionBindings,
) -> Result<(), CodegenErrors> {
    let resource = bindings
        .input(&input.id)
        .and_then(|binding| binding.resource())
        .ok_or_else(|| failure(input, "LUT has no protected resource binding"))?;
    if !resource
        .path()
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("cube"))
    {
        return Err(failure(input, "LUT resource must use the .cube format"));
    }
    let bytes = resource
        .read_verified_bounded(cube::MAX_FILE_BYTES)
        .map_err(|error| {
            if error.kind == ArtifactErrorKind::ResourceLimit {
                limit(
                    input,
                    format!("LUT resource exceeds the byte budget: {error}"),
                )
            } else {
                failure(input, format!("cannot verify LUT resource: {error}"))
            }
        })?;
    cube::validate(&bytes, expected).map_err(|error| {
        if error.limit {
            limit(input, error.message)
        } else {
            failure(input, error.message)
        }
    })
}

fn failure(input: &ResolvedInput, message: impl Into<String>) -> CodegenErrors {
    error(input, "LUT_RESOURCE_INVALID", message)
}

fn limit(input: &ResolvedInput, message: impl Into<String>) -> CodegenErrors {
    error(input, "LUT_RESOURCE_LIMIT", message)
}

fn error(input: &ResolvedInput, code: &'static str, message: impl Into<String>) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        code,
        Some(input.id.to_string()),
        message,
    ))
}
