use super::raw_with_types;
use crate::program::expression::{
    CoreInstructionKind, FunctionEffect, PrimitiveType, TypeEnvironment,
};

#[test]
fn declared_input_types_precede_later_value_first_uses_canonically() {
    for (source, expected) in [
        ("{ let ignored = 1; label }", &["text", "int"][..]),
        (
            "if true { \"literal\" } else { label }",
            &["text", "bool"][..],
        ),
    ] {
        let (first, functions) = compiled(source);
        let (repeated, _) = compiled(source);
        assert_eq!(type_names(&first), expected);
        assert_eq!(first.types.entries(), repeated.types.entries());
        assert_eq!(digest(&first), digest(&repeated));
        assert!(super::super::verify(first, functions.registry(), &[]).is_ok());
    }
}

#[test]
fn input_used_only_in_a_branch_keeps_declaration_first_order() {
    let (program, _) = compiled("if true { \"literal\" } else { label }");
    assert!(!program.blocks[0]
        .instructions
        .iter()
        .any(|value| matches!(value.kind, CoreInstructionKind::Input(_))));
    assert!(program.blocks[1..]
        .iter()
        .flat_map(|block| &block.instructions)
        .any(|value| matches!(value.kind, CoreInstructionKind::Input(_))));
}

fn compiled(source: &str) -> (super::super::CoreProgram, super::FunctionMap) {
    let types = TypeEnvironment::from([("label".to_owned(), PrimitiveType::Text.into())]);
    raw_with_types(source, &types)
}

fn type_names(program: &super::super::CoreProgram) -> Vec<String> {
    program
        .types
        .entries()
        .iter()
        .map(|entry| entry.kind().value_type().unwrap().to_string())
        .collect()
}

fn digest(program: &super::super::CoreProgram) -> super::super::CoreDigest {
    super::super::closure_digest(&[], &[], &[], FunctionEffect::Pure, false, program)
}
