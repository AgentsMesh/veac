use super::*;

#[test]
fn invalid_text_errors_keep_the_plan_error_code_and_kind() {
    let error = TextError::invalid("invalid authored text");
    assert_eq!(error.code, "TEXT_PLAN_INVALID");
    assert_eq!(error.message, "invalid authored text");
    assert_eq!(kind(error.code), CodegenErrorKind::InvalidPlan);
}

#[test]
fn borrowed_font_binding_messages_keep_the_resource_error_contract() {
    let error = TextError::binding("missing font binding");
    assert_eq!(error.code, "TEXT_FONT_BINDING_INVALID");
    assert_eq!(error.message, "missing font binding");
    assert_eq!(kind(error.code), CodegenErrorKind::InvalidResourceBinding);
}
