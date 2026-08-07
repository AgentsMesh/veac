use super::{empty_registry, raw, raw_with_types, verify_error};
use crate::program::expression::{
    BlockId, CoreCallTarget, CoreInstructionKind, CoreTerminator, CoreType, CoreTypeEntry,
    CoreTypeId, DependencyMask, FunctionId, InputId, Stage, TypeEnvironment, ValueId, ValueType,
};

fn core_code(program: super::super::CoreProgram) -> &'static str {
    verify_error(program, &crate::program::expression::FunctionMap::new()).code()
}

#[test]
fn rejects_version_block_and_value_identity_corruption() {
    let (mut version, _) = raw("1");
    version.version += 1;
    assert_eq!(core_code(version), "EXPRESSION_CORE_VERIFY");

    let (mut block, _) = raw("1");
    block.blocks[0].id = BlockId::new(1);
    assert_eq!(core_code(block), "EXPRESSION_CORE_VERIFY");

    let (mut duplicate, _) = raw("1 + 2");
    duplicate.blocks[0].instructions[1].id = ValueId::new(0);
    assert_eq!(core_code(duplicate), "EXPRESSION_CORE_VERIFY");
}

#[test]
fn rejects_missing_forward_and_non_dominating_operands() {
    let (mut missing, _) = raw("1 + 2");
    let root = missing.blocks[0].instructions.last_mut().unwrap();
    let CoreInstructionKind::Arithmetic { left, .. } = &mut root.kind else {
        unreachable!()
    };
    *left = ValueId::new(99);
    assert_eq!(core_code(missing), "EXPRESSION_CORE_VERIFY");

    let (mut forward, _) = raw("1 + 2");
    let root_id = forward.blocks[0].instructions.last().unwrap().id;
    let root = forward.blocks[0].instructions.last_mut().unwrap();
    let CoreInstructionKind::Arithmetic { left, .. } = &mut root.kind else {
        unreachable!()
    };
    *left = root_id;
    assert_eq!(core_code(forward), "EXPRESSION_CORE_VERIFY");

    let (mut dominance, _) = raw("if true { 1 } else { 2 }");
    let then_value = dominance.blocks[1].instructions[0].id;
    let CoreTerminator::Jump { arguments, .. } = &mut dominance.blocks[2].terminator else {
        unreachable!()
    };
    arguments[0] = then_value;
    assert_eq!(core_code(dominance), "EXPRESSION_CORE_VERIFY");
}

#[test]
fn rejects_control_flow_target_arity_condition_and_cycles() {
    let (mut target, _) = raw("if true { 1 } else { 2 }");
    let CoreTerminator::Branch { then_target, .. } = &mut target.blocks[0].terminator else {
        unreachable!()
    };
    *then_target = BlockId::new(99);
    assert_eq!(core_code(target), "EXPRESSION_CORE_VERIFY");

    let (mut arity, _) = raw("if true { 1 } else { 2 }");
    let CoreTerminator::Jump { arguments, .. } = &mut arity.blocks[1].terminator else {
        unreachable!()
    };
    arguments.clear();
    assert_eq!(core_code(arity), "EXPRESSION_CORE_VERIFY");

    let (mut condition, _) = raw("if 1 == 1 { 1 } else { 2 }");
    let CoreTerminator::Branch { condition: id, .. } = &mut condition.blocks[0].terminator else {
        unreachable!()
    };
    *id = ValueId::new(0);
    assert_eq!(core_code(condition), "EXPRESSION_CORE_VERIFY");

    let (mut cycle, _) = raw("if true { 1 } else { 2 }");
    let CoreTerminator::Jump { target, .. } = &mut cycle.blocks[1].terminator else {
        unreachable!()
    };
    *target = BlockId::new(0);
    assert_eq!(core_code(cycle), "EXPRESSION_CORE_VERIFY");
}

#[test]
fn rejects_return_input_call_and_metadata_contract_corruption() {
    let (mut result, _) = raw("1");
    result.types.entries.push(CoreTypeEntry {
        id: CoreTypeId::new(1),
        kind: CoreType::Value(ValueType::parse("time").unwrap()),
    });
    result.result_type = CoreTypeId::new(1);
    assert_eq!(core_code(result), "EXPRESSION_CORE_VERIFY");

    let types = [("input".to_owned(), ValueType::parse("scalar").unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let (mut input, _) = raw_with_types("input", &types);
    input.blocks[0].instructions[0].kind = CoreInstructionKind::Input(InputId::new(8));
    assert_eq!(core_code(input), "EXPRESSION_CORE_VERIFY");

    let (mut call, _) = raw("min(1, 2)");
    let CoreInstructionKind::Call { target, .. } = &mut call.blocks[0].instructions[2].kind else {
        unreachable!()
    };
    *target = CoreCallTarget::User(FunctionId::from_bytes([7; 32]));
    assert!(super::super::verify(call, &empty_registry(), &[]).is_err());

    let (mut metadata, _) = raw("1");
    metadata.blocks[0].instructions[0].metadata.leaf_stage = Stage::Temporal;
    assert_eq!(core_code(metadata), "EXPRESSION_CORE_VERIFY");
}

#[test]
fn rejects_shape_or_leaf_dependency_corruption_independently() {
    let types = [("input".to_owned(), ValueType::parse("scalar").unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let (mut shape, _) = raw_with_types("input", &types);
    shape.blocks[0].instructions[0].metadata.shape_dependencies = DependencyMask::default();
    assert_eq!(core_code(shape), "EXPRESSION_CORE_VERIFY");

    let (mut shape_origin, _) = raw_with_types("input", &types);
    shape_origin.blocks[0].instructions[0]
        .metadata
        .shape_dependencies = DependencyMask::input_leaf(InputId::new(0));
    assert_eq!(core_code(shape_origin), "EXPRESSION_CORE_VERIFY");

    let (mut leaf, _) = raw_with_types("input", &types);
    leaf.blocks[0].instructions[0].metadata.leaf_dependencies = DependencyMask::default();
    assert_eq!(core_code(leaf), "EXPRESSION_CORE_VERIFY");

    let (mut leaf_origin, _) = raw_with_types("input", &types);
    leaf_origin.blocks[0].instructions[0]
        .metadata
        .leaf_dependencies = DependencyMask::input_shape(InputId::new(0));
    assert_eq!(core_code(leaf_origin), "EXPRESSION_CORE_VERIFY");

    let (mut join, _) = raw_with_types("if input == 1.0 { input } else { 2.0 }", &types);
    join.blocks[3].parameters[0].metadata.shape_dependencies = DependencyMask::default();
    assert_eq!(core_code(join), "EXPRESSION_CORE_VERIFY");

    let (mut join, _) = raw_with_types("if input == 1.0 { input } else { 2.0 }", &types);
    join.blocks[3].parameters[0].metadata.leaf_dependencies = DependencyMask::default();
    assert_eq!(core_code(join), "EXPRESSION_CORE_VERIFY");
}
