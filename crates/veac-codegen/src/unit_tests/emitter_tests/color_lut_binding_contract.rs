use std::fs;

use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::{CodegenErrorKind, CodegenErrors};
use veac_plan::canonical::{LutInterpolation, MaterialKind};
use veac_plan::{ResolvedColorStage, ResolvedInputKind};

use super::support::{
    bindings_with_original, emit_video_command, graded_project, lut_fixture, output_bindings,
    resolved,
};

#[test]
fn binding_format_identity_presence_and_byte_budget_are_enforced() {
    let plan = resolved(&graded_project(
        MaterialKind::Lut3d,
        LutInterpolation::Tetrahedral,
    ));
    let resource = plan
        .inputs
        .iter()
        .find(|input| matches!(input.kind, ResolvedInputKind::Resource { .. }))
        .unwrap();
    let temp = tempfile::tempdir().unwrap();

    let wrong_extension = temp.path().join("look.dat");
    fs::copy(lut_fixture(MaterialKind::Lut3d), &wrong_extension).unwrap();
    let local = bindings_with_original(&plan, &resource.id, wrong_extension);
    assert_code(
        emit_video_command(&plan, &local),
        "LUT_RESOURCE_INVALID",
        "extension",
    );

    let mismatch = temp.path().join("mismatch.cube");
    fs::write(&mismatch, "LUT_3D_SIZE 2\n").unwrap();
    let local = bindings_with_original(&plan, &resource.id, mismatch);
    assert_code(
        emit_video_command(&plan, &local),
        "LUT_RESOURCE_INVALID",
        "identity",
    );

    let oversized = temp.path().join("oversized.cube");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    let local = bindings_with_original(&plan, &resource.id, oversized);
    assert_code(
        emit_video_command(&plan, &local),
        "LUT_RESOURCE_LIMIT",
        "bytes",
    );

    assert_code(
        emit_video_command(&plan, &ExecutionBindings::default()),
        "LUT_RESOURCE_INVALID",
        "missing binding",
    );
}

#[test]
fn an_unreferenced_lut_needs_no_binding_or_validation() {
    let mut plan = resolved(&graded_project(
        MaterialKind::Lut3d,
        LutInterpolation::Tetrahedral,
    ));
    let pipeline = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap();
    pipeline
        .stages
        .retain(|stage| !matches!(stage, ResolvedColorStage::Lut { .. }));
    let mut local = output_bindings(&plan);
    for input in &plan.inputs {
        if matches!(input.kind, ResolvedInputKind::Media { .. }) {
            local
                .bind_original(input, format!("/tmp/{}.bin", input.id).into())
                .unwrap();
        }
    }
    emit_video_command(&plan, &local).unwrap();
}

fn assert_code(result: Result<impl Sized, CodegenErrors>, code: &str, name: &str) {
    let error = match result {
        Ok(_) => panic!("{name} unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::InvalidResourceBinding,
        "{name}"
    );
    assert_eq!(error.diagnostics()[0].code, code, "{name}: {error}");
}
