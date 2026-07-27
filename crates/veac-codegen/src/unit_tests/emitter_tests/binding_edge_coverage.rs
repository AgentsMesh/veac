use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::emit_all;
use veac_plan::PlanInputId;

use super::support::{bindings, fixture, resolved};

#[test]
fn typed_bindings_report_missing_and_extra_plan_inputs() {
    let plan = resolved(&fixture());
    assert_code(
        emit_all(&plan, &ExecutionBindings::default()).unwrap_err(),
        "INPUT_BINDING_MISSING",
    );

    let mut typed = bindings(&plan);
    let mut extra = plan.inputs[0].clone();
    extra.id = PlanInputId::new("pin_extra").unwrap();
    typed
        .bind_original(&extra, "/tmp/extra.bin".into())
        .unwrap();
    assert_code(
        emit_all(&plan, &typed).unwrap_err(),
        "RESOURCE_BINDING_INVALID",
    );
}

fn assert_code(error: veac_codegen::emitter::CodegenErrors, expected: &str) {
    assert_eq!(error.diagnostics()[0].code, expected);
}
