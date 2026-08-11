use super::domain_fixture::{compiled, project_program, resource_source_program, result_domain};
use super::empty_registry;
use crate::program::expression::{
    CoreInstructionKind, CoreType, CoreValueMetadata, Effect, Stage, ValueId, ValueType,
};
use crate::program::{DomainOpsetVersion, DomainRegistryDigest, DomainType};

fn failure(program: super::super::CoreProgram) -> String {
    super::super::verify(program, &empty_registry(), &[]).unwrap_err().message().to_owned()
}

#[test]
fn core_v8_pins_verified_domain_registry_identity() {
    let program = project_program();
    assert_eq!(program.version(), 7);
    assert_eq!(result_domain(&program), DomainType::Project);
    assert_eq!(
        program.domain_opset(),
        crate::program::DomainOperationRegistry::standard().version()
    );

    let mut wrong_version = program.clone();
    wrong_version.domain_opset = DomainOpsetVersion::from_raw(99);
    assert!(failure(wrong_version).contains("opset"));

    let mut wrong_digest = program;
    wrong_digest.domain_registry_digest = DomainRegistryDigest::from_bytes([7; 32]);
    assert!(failure(wrong_digest).contains("registry digest"));
}

#[test]
fn rejects_unknown_opcode_wrong_instruction_kind_and_arity() {
    let mut unknown = project_program();
    let CoreInstructionKind::DomainConstruct { opcode, .. } =
        &mut unknown.blocks[0].instructions[2].kind
    else {
        unreachable!()
    };
    *opcode = 0xffff;
    assert!(failure(unknown).contains("unknown domain operation opcode"));

    let mut wrong_kind = project_program();
    wrong_kind.blocks[0].instructions[2].kind = CoreInstructionKind::GraphEmit {
        opcode: crate::program::DomainOperationId::Canvas.opcode(),
        operands: vec![ValueId::new(0), ValueId::new(1)],
    };
    assert!(failure(wrong_kind).contains("wrong Core instruction kind"));

    let mut arity = project_program();
    let CoreInstructionKind::DomainConstruct { operands, .. } =
        &mut arity.blocks[0].instructions[2].kind
    else {
        unreachable!()
    };
    operands.pop();
    assert!(failure(arity).contains("operand arity"));
}

#[test]
fn rejects_operand_and_result_types_that_disagree_with_contract() {
    let mut operand = project_program();
    let CoreInstructionKind::GraphEmit { operands, .. } =
        &mut operand.blocks[0].instructions[8].kind
    else {
        unreachable!()
    };
    operands[0] = ValueId::new(0);
    assert!(failure(operand).contains("operand `key` type"));

    let mut result = project_program();
    result.types.entries[1].kind = CoreType::Value(ValueType::parse("bool").unwrap());
    let message = failure(result);
    assert!(message.contains("expected Canvas"), "{message}");
}

#[test]
fn pure_construct_and_graph_emit_have_exact_build_stage_metadata() {
    let program = project_program();
    let canvas = &program.blocks[0].instructions[2].metadata;
    assert_eq!(canvas.effect(), Effect::Pure);
    assert_eq!(canvas.shape_stage(), Stage::Build);
    assert_eq!(canvas.leaf_stage(), Stage::Const);
    let project = &program.blocks[0].instructions[6].metadata;
    assert_eq!(project.effect(), Effect::GraphEmit);
    assert_eq!(project.shape_stage(), Stage::Build);

    for corrupt in [
        CoreValueMetadata::constant(),
        CoreValueMetadata::pure(Stage::Temporal, Default::default(), Default::default()),
    ] {
        let mut program = project_program();
        program.blocks[0].instructions[2].metadata = corrupt;
        assert!(failure(program).contains("domain instruction metadata"));
    }
}

#[test]
fn opcode_and_registry_identity_are_part_of_core_digest() {
    let program = project_program();
    let original = super::super::closure_digest(
        &[],
        &[],
        &[],
        crate::program::expression::FunctionEffect::Pure,
        false,
        &program,
    );
    let mut mutated = program.clone();
    let CoreInstructionKind::DomainConstruct { opcode, .. } =
        &mut mutated.blocks[0].instructions[2].kind
    else {
        unreachable!()
    };
    *opcode = crate::program::DomainOperationId::FramesPerSecond.opcode();
    assert_ne!(
        original,
        super::super::closure_digest(
            &[],
            &[],
            &[],
            crate::program::expression::FunctionEffect::Pure,
            false,
            &mutated,
        )
    );
    let mut identity = program;
    identity.domain_registry_digest = DomainRegistryDigest::from_bytes([1; 32]);
    assert_ne!(
        original,
        super::super::closure_digest(
            &[],
            &[],
            &[],
            crate::program::expression::FunctionEffect::Pure,
            false,
            &identity,
        )
    );
}

#[test]
fn raw_resource_emit_and_media_reference_verify_against_v8_contracts() {
    let program = resource_source_program();
    assert_eq!(result_domain(&program), DomainType::Source);
    assert_eq!(program.domain_opset(), DomainOpsetVersion::V8);
    let identity = &program.blocks[0].instructions[3];
    assert!(matches!(
        identity.kind(),
        CoreInstructionKind::DomainConstruct { opcode, operands }
            if *opcode == crate::program::DomainOperationId::Sha256.opcode()
                && operands == &[ValueId::new(2)]
    ));
    let image = &program.blocks[0].instructions[4];
    assert!(matches!(
        image.kind(),
        CoreInstructionKind::GraphEmit { opcode, operands }
            if *opcode == crate::program::DomainOperationId::ImageResource.opcode()
                && operands == &[ValueId::new(0), ValueId::new(1), ValueId::new(3)]
    ));
    assert_eq!(image.metadata().effect(), Effect::GraphEmit);
    assert_eq!(image.metadata().shape_stage(), Stage::Build);
    let media = &program.blocks[0].instructions[5];
    assert!(matches!(
        media.kind(),
        CoreInstructionKind::DomainConstruct { opcode, operands }
            if *opcode == crate::program::DomainOperationId::Media.opcode()
                && operands == &[ValueId::new(4)]
    ));
    assert_eq!(media.metadata().effect(), Effect::GraphEmit);
    compiled(program);
}

#[test]
fn raw_resource_contract_rejects_path_and_instruction_corruption() {
    let mut path = resource_source_program();
    let CoreInstructionKind::GraphEmit { operands, .. } = &mut path.blocks[0].instructions[4].kind
    else {
        unreachable!()
    };
    operands[1] = ValueId::new(0);
    assert!(failure(path).contains("operand `path` type"));

    let mut kind = resource_source_program();
    kind.blocks[0].instructions[4].kind = CoreInstructionKind::DomainConstruct {
        opcode: crate::program::DomainOperationId::ImageResource.opcode(),
        operands: vec![ValueId::new(0), ValueId::new(1), ValueId::new(3)],
    };
    assert!(failure(kind).contains("wrong Core instruction kind"));

    let mut identity = resource_source_program();
    let CoreInstructionKind::GraphEmit { operands, .. } =
        &mut identity.blocks[0].instructions[4].kind
    else {
        unreachable!()
    };
    operands[2] = ValueId::new(1);
    assert!(failure(identity).contains("operand `identity` type"));
}
