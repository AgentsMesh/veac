use super::*;
use crate::program::domain_system::{catalog, validation};

fn error(value: &DomainOperationContract) -> &'static str {
    validation::contract(value).unwrap_err().code()
}

#[test]
fn inconsistent_instruction_result_and_temporal_fields_fail_closed() {
    let mut instruction = catalog::contract(DomainOperationId::Canvas);
    instruction.semantics.effect = Effect::GraphEmit;
    assert_eq!(error(&instruction), "DOMAIN_OPERATION_EFFECT");

    let mut result = catalog::contract(DomainOperationId::Canvas);
    result.result = DomainValueShape::primitive(crate::program::expression::PrimitiveType::Integer);
    assert_eq!(error(&result), "DOMAIN_OPERATION_RESULT");

    let mut temporal = catalog::contract(DomainOperationId::Canvas);
    temporal.max_stage = Stage::Temporal;
    temporal.temporal_lowering = None;
    assert_eq!(error(&temporal), "DOMAIN_OPERATION_TEMPORAL");
}
