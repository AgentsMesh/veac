use std::sync::Arc;

use crate::program::expression::{
    BlockId, CoreBlock, CoreBuildInputId, CoreInput, CoreInputIdentity, CoreInstruction,
    CoreInstructionKind, CoreNominalDefinition, CoreProgram, CoreTerminator, CoreType,
    CoreTypeEntry, CoreTypeId, CoreTypeTable, CoreValueMetadata, FunctionMap, InputId,
    PrimitiveType, Stage, ValueId, ValueType, CORE_VERSION,
};
use crate::program::{
    FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind,
};

#[test]
fn rejects_nominal_function_wrapper_marked_as_trusted_input() {
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "CallbackBox",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "callback",
            ValueType::function(
                Vec::new(),
                ValueType::primitive(PrimitiveType::Integer),
                crate::program::expression::FunctionEffect::Pure,
            )
            .unwrap(),
        )])),
    ));
    let type_id = CoreTypeId::new(0);
    let input_id = InputId::new(0);
    let domain = crate::program::DomainOperationRegistry::standard();
    let program = CoreProgram {
        version: CORE_VERSION,
        domain_opset: domain.version(),
        domain_registry_digest: domain.digest(),
        entry: BlockId::new(0),
        types: CoreTypeTable::new(vec![CoreTypeEntry {
            id: type_id,
            kind: CoreType::Value(ValueType::nominal(definition.type_ref().clone())),
        }]),
        nominal_definitions: vec![CoreNominalDefinition::new(definition, 0..1)],
        inputs: vec![CoreInput {
            id: input_id,
            name: "box".into(),
            identity: CoreInputIdentity::Build(CoreBuildInputId::for_symbol("box")),
            type_id,
            trusted_function: true,
            callable: None,
            span: 0..1,
        }],
        local_slots: Vec::new(),
        closure_definitions: Vec::new(),
        blocks: vec![CoreBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![CoreInstruction {
                id: ValueId::new(0),
                kind: CoreInstructionKind::Input(input_id),
                type_id,
                metadata: CoreValueMetadata::input(Stage::Build, input_id),
                span: 0..1,
            }],
            terminator: CoreTerminator::Return {
                value: ValueId::new(0),
                span: 0..1,
            },
        }],
        result_type: type_id,
    };
    let functions = FunctionMap::new();
    let error =
        super::super::verify_with_input_trust(program, functions.registry(), &[], &|name| {
            name == "box"
        })
        .unwrap_err();
    assert!(error.message().contains("function trust does not match"));
}
