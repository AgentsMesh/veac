use super::callable_mismatch;
use crate::program::expression::core::{
    CoreBuildInputId, CoreInput, CoreInputIdentity, CoreTypeId, InputId,
};

#[test]
fn callable_contract_failure_keeps_input_name_and_source_span() {
    let input = CoreInput {
        id: InputId::new(0),
        name: "callback".to_owned(),
        identity: CoreInputIdentity::Build(CoreBuildInputId::for_symbol("callback")),
        type_id: CoreTypeId::new(0),
        trusted_function: true,
        callable: None,
        span: 7..15,
    };
    let error = callable_mismatch(&input);
    assert_eq!(error.code(), "EXPRESSION_RUNTIME_CONTRACT");
    assert!(error.message().contains("`callback`"));
    assert_eq!(error.span(), 7..15);
}
