use veac_plan::canonical::LutInterpolation;
use veac_plan::{ResolvedInputKind, ResolvedLut, ResolvedLutKind};

use super::error::{diagnostic, CodegenErrorKind};
use super::{process_owner::ProcessOwner, BackendFilterEscape, CodegenErrors, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    input: String,
    value: &ResolvedLut,
) -> Result<String, CodegenErrors> {
    let Some(resolved) = context
        .plan
        .inputs
        .iter()
        .find(|input| input.id == value.input_id)
    else {
        return Err(resource_error(owner, "LUT resolved input is missing"));
    };
    if !matches!(resolved.kind, ResolvedInputKind::Resource { .. }) {
        return Err(resource_error(owner, "LUT input is not a resource"));
    }
    let path = context
        .bindings
        .input(&value.input_id)
        .and_then(|binding| binding.resource())
        .map(|resource| resource.path())
        .filter(|path| path.to_str().is_some())
        .map(ToOwned::to_owned)
        .ok_or_else(|| resource_error(owner, "LUT binding is missing or is not UTF-8"))?;
    let (filter_name, interpolation) = options(value)
        .ok_or_else(|| unsupported(owner, "LUT interpolation does not match its dimension"))?;
    let token = context.filter_file(path, BackendFilterEscape::FilterValue);
    Ok(context.graph.filter(
        &[&input],
        format!("{filter_name}=file={token}:interp={interpolation}"),
        "lutv",
    ))
}

fn options(value: &ResolvedLut) -> Option<(&'static str, &'static str)> {
    let interpolation = match (value.kind, value.interpolation) {
        (ResolvedLutKind::OneDimensional, LutInterpolation::Nearest) => "nearest",
        (ResolvedLutKind::OneDimensional, LutInterpolation::Linear) => "linear",
        (ResolvedLutKind::OneDimensional, LutInterpolation::Cosine) => "cosine",
        (ResolvedLutKind::OneDimensional, LutInterpolation::Cubic) => "cubic",
        (ResolvedLutKind::OneDimensional, LutInterpolation::Spline) => "spline",
        (ResolvedLutKind::ThreeDimensional, LutInterpolation::Nearest) => "nearest",
        (ResolvedLutKind::ThreeDimensional, LutInterpolation::Trilinear) => "trilinear",
        (ResolvedLutKind::ThreeDimensional, LutInterpolation::Tetrahedral) => "tetrahedral",
        (ResolvedLutKind::ThreeDimensional, LutInterpolation::Pyramid) => "pyramid",
        (ResolvedLutKind::ThreeDimensional, LutInterpolation::Prism) => "prism",
        _ => return None,
    };
    let name = match value.kind {
        ResolvedLutKind::OneDimensional => "lut1d",
        ResolvedLutKind::ThreeDimensional => "lut3d",
    };
    Some((name, interpolation))
}

fn unsupported(owner: ProcessOwner<'_>, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::UnsupportedColorProcessing,
        "COLOR_PROCESSING_UNSUPPORTED",
        Some(owner.id()),
        message,
    ))
}

fn resource_error(owner: ProcessOwner<'_>, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        "COLOR_RESOURCE_INVALID",
        Some(owner.id()),
        message,
    ))
}
