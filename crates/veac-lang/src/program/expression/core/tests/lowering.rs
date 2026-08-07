use super::{raw, raw_with_types};
use crate::program::expression::{
    BlockId, CoreInstructionKind, CoreTerminator, InputId, TypeEnvironment, ValueType, CORE_VERSION,
};

#[test]
fn conditional_lowers_to_explicit_blocks_jumps_and_join_parameter() {
    let (program, _) = raw("if true { 1 } else { 2 }");
    assert_eq!(program.version(), CORE_VERSION);
    assert_eq!(program.entry(), BlockId::new(0));
    assert_eq!(program.blocks().len(), 4);
    assert!(matches!(
        program.blocks()[0].terminator(),
        CoreTerminator::Branch { .. }
    ));
    assert!(matches!(
        program.blocks()[1].terminator(),
        CoreTerminator::Jump { .. }
    ));
    assert!(matches!(
        program.blocks()[2].terminator(),
        CoreTerminator::Jump { .. }
    ));
    assert_eq!(program.blocks()[3].parameters().len(), 1);
    assert!(matches!(
        program.blocks()[3].terminator(),
        CoreTerminator::Return { .. }
    ));
}

#[test]
fn locals_disappear_into_value_ids_and_logical_operators_branch() {
    let (locals, _) = raw("{ let value = 1; value + value }");
    assert_eq!(locals.blocks().len(), 1);
    assert_eq!(locals.blocks()[0].instructions().len(), 2);

    for source in ["true && false", "true || false"] {
        let (program, _) = raw(source);
        assert_eq!(program.blocks().len(), 4);
        assert!(matches!(
            program.blocks()[0].terminator(),
            CoreTerminator::Branch { .. }
        ));
    }
}

#[test]
fn declared_inputs_are_dense_deduplicated_slots() {
    let types = [("input".to_owned(), ValueType::parse("scalar").unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let (program, _) = raw_with_types("input + input", &types);
    assert_eq!(program.inputs().len(), 1);
    assert_eq!(program.inputs()[0].id(), InputId::new(0));
    assert_eq!(program.inputs()[0].name(), "input");
    let loads = program.blocks()[0]
        .instructions()
        .iter()
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::Input(id) => Some(*id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(loads, [InputId::new(0), InputId::new(0)]);
}
