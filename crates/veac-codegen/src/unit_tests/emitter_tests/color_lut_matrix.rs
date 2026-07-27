use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::{LutInterpolation, MaterialKind};
use veac_plan::ResolvedInputKind;

use super::support::{
    bindings, bindings_with_original, emit_video_command, graded_project, lut_fixture, resolved,
};

#[test]
fn every_supported_lut_interpolation_emits_its_exact_backend_option() {
    let cases = [
        (
            MaterialKind::Lut1d,
            LutInterpolation::Nearest,
            "lut1d",
            "nearest",
        ),
        (
            MaterialKind::Lut1d,
            LutInterpolation::Linear,
            "lut1d",
            "linear",
        ),
        (
            MaterialKind::Lut1d,
            LutInterpolation::Cosine,
            "lut1d",
            "cosine",
        ),
        (
            MaterialKind::Lut1d,
            LutInterpolation::Cubic,
            "lut1d",
            "cubic",
        ),
        (
            MaterialKind::Lut1d,
            LutInterpolation::Spline,
            "lut1d",
            "spline",
        ),
        (
            MaterialKind::Lut3d,
            LutInterpolation::Nearest,
            "lut3d",
            "nearest",
        ),
        (
            MaterialKind::Lut3d,
            LutInterpolation::Trilinear,
            "lut3d",
            "trilinear",
        ),
        (
            MaterialKind::Lut3d,
            LutInterpolation::Tetrahedral,
            "lut3d",
            "tetrahedral",
        ),
        (
            MaterialKind::Lut3d,
            LutInterpolation::Pyramid,
            "lut3d",
            "pyramid",
        ),
        (
            MaterialKind::Lut3d,
            LutInterpolation::Prism,
            "lut3d",
            "prism",
        ),
    ];
    for (kind, interpolation, filter, option) in cases {
        let plan = resolved(&graded_project(kind, interpolation));
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        let expected = format!(
            "{filter}=file={}:interp={option}",
            lut_fixture(kind).display()
        );
        assert!(graph.contains(&expected), "missing {expected}: {graph}");
    }
}

#[test]
fn malformed_resolved_lut_references_fail_as_invalid_plans() {
    let base = resolved(&graded_project(
        MaterialKind::Lut3d,
        LutInterpolation::Tetrahedral,
    ));
    let resource = base
        .inputs
        .iter()
        .position(|input| matches!(input.kind, ResolvedInputKind::Resource { .. }))
        .unwrap();

    let mut wrong_kind = base.clone();
    wrong_kind.inputs[resource].kind = ResolvedInputKind::Media {
        material_kind: MaterialKind::Video,
    };
    assert_plan_error(&wrong_kind, bindings(&wrong_kind));

    let mut missing = base.clone();
    missing.inputs.remove(resource);
    assert_plan_error(&missing, bindings(&missing));
}

#[cfg(unix)]
#[test]
fn non_utf8_lut_binding_fails_before_filter_serialization() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let plan = resolved(&graded_project(
        MaterialKind::Lut3d,
        LutInterpolation::Tetrahedral,
    ));
    let resource = plan
        .inputs
        .iter()
        .find(|input| matches!(input.kind, ResolvedInputKind::Resource { .. }))
        .unwrap();
    let local = bindings_with_original(
        &plan,
        &resource.id,
        std::path::PathBuf::from(OsString::from_vec(vec![0xff])),
    );
    assert_resource_error(&plan, local, "LUT_RESOURCE_INVALID");
}

fn assert_resource_error(
    plan: &veac_plan::ResolvedRenderPlan,
    bindings: ExecutionBindings,
    code: &str,
) {
    let error = emit_video_command(plan, &bindings).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::InvalidResourceBinding
    );
    assert_eq!(error.diagnostics()[0].code, code);
}

fn assert_plan_error(plan: &veac_plan::ResolvedRenderPlan, bindings: ExecutionBindings) {
    let error = emit_video_command(plan, &bindings).unwrap_err();
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind == CodegenErrorKind::InvalidPlan
            && diagnostic.code == "PLAN_COLOR_PIPELINE_INVALID"
    }));
}
