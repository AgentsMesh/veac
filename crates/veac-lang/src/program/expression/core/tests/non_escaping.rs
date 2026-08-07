use super::{raw, verify_error};
use crate::program::expression::core::{closure_digest, CoreProgram};

#[test]
fn non_escaping_closure_cannot_return_or_flow_into_a_list() {
    let (mut returned, functions) = raw("fn(value: int) -> int effect pure { value }");
    mark_non_escaping(&mut returned);
    let error = verify_error(returned, &functions);
    assert!(error
        .message()
        .contains("cannot flow through control or Return"));

    let (mut stored, functions) = raw("[fn(value: int) -> int effect pure { value }]");
    mark_non_escaping(&mut stored);
    let error = verify_error(stored, &functions);
    assert!(error.message().contains("only be a Collection callback"));
}

#[test]
fn non_escaping_closure_cannot_be_normally_invoked_or_left_unused() {
    let (mut invoked, functions) = raw("(fn(value: int) -> int effect pure { value })(1)");
    mark_non_escaping(&mut invoked);
    let error = verify_error(invoked, &functions);
    assert!(error.message().contains("only be a Collection callback"));

    let (mut unused, functions) =
        raw("{ let callback = fn(value: int) -> int effect pure { value }; 1 }");
    mark_non_escaping(&mut unused);
    let error = verify_error(unused, &functions);
    assert!(error
        .message()
        .contains("exactly one Collection callback use"));
}

fn mark_non_escaping(program: &mut CoreProgram) {
    let definition = &program.closure_definitions[0];
    let digest = closure_digest(
        &definition.parameter_types,
        &definition.parameter_stages,
        &definition.capture_types,
        definition.effect,
        true,
        &definition.body,
    );
    program.closure_definitions[0].non_escaping = true;
    program.closure_definitions[0].digest = digest;
}
