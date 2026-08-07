use super::*;
use crate::program::expression::core::verify::control::ControlFlow;
use crate::program::expression::core::verify::definitions::Definitions;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{
    CoreType, CoreTypeEntry, CoreTypeId, PrimitiveType, Stage, ValueType,
};
use crate::program::TypeRegistry;

#[test]
fn argument_requires_the_parameter_to_cover_every_metadata_axis() {
    let parameter = CoreValueMetadata::constant();
    let argument = CoreValueMetadata::pure(Stage::Temporal, Default::default(), Default::default());
    assert!(super::argument(&argument, &parameter, 0..1).is_err());
    assert!(super::argument(&argument, &argument, 0..1).is_ok());
}

#[test]
fn parameters_require_a_declared_value_type() {
    let (mut program, _) = raw("if true { 1 } else { 2 }");
    let definitions = Definitions::collect(&program).unwrap();
    let control = ControlFlow::analyze(&program).unwrap();
    let parameter = program
        .blocks
        .iter_mut()
        .find_map(|block| block.parameters.first_mut())
        .unwrap();
    parameter.type_id = CoreTypeId::new(99);
    assert!(
        super::parameters(&program, &definitions, &control, &TypeRegistry::default())
            .unwrap_err()
            .message()
            .contains("must have a value type")
    );
}

#[test]
fn match_payload_requires_a_closed_field_contract() {
    let mut program = crate::program::expression::core::tests::nominal_fixture::matching();
    program.blocks[0].instructions[1].metadata = CoreValueMetadata::constant();
    let callable = ValueType::function(
        Vec::new(),
        PrimitiveType::Integer.into(),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let type_id = CoreTypeId::new(program.types.entries.len() as u32);
    program.types.entries.push(CoreTypeEntry {
        id: type_id,
        kind: CoreType::Value(callable),
    });
    program.blocks[1].parameters[0].type_id = type_id;
    let definitions = Definitions::collect(&program).unwrap();
    let control = ControlFlow::analyze(&program).unwrap();
    assert!(
        super::parameters(&program, &definitions, &control, &TypeRegistry::default())
            .unwrap_err()
            .message()
            .contains("closed field metadata contract")
    );
}
