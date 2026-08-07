use std::mem::size_of;

use crate::program::expression::{
    compile_expression, CoreInstructionKind, ExpressionContext, TypeEnvironment, ValueId,
};

#[test]
fn every_instruction_operand_vector_has_an_exact_retained_delta() {
    assert_vector("min(1, 2)", |kind| {
        matches!(kind, CoreInstructionKind::Call { .. })
    });
    assert_vector(
        "{ let first = 1; let second = 2; fn() -> int effect pure { first + second } }",
        |kind| matches!(kind, CoreInstructionKind::Closure { .. }),
    );
    assert_vector("(fn(value: int) -> int effect pure { value })(1)", |kind| {
        matches!(kind, CoreInstructionKind::Invoke { .. })
    });
    assert_vector("[1, 2]", |kind| {
        matches!(kind, CoreInstructionKind::List { .. })
    });
    assert_vector("(1, 2)", |kind| {
        matches!(kind, CoreInstructionKind::Tuple { .. })
    });
}

fn assert_vector(source: &str, select: impl Fn(&CoreInstructionKind) -> bool) {
    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    let mut program = compiled.core().clone();
    let before = program.retained_bytes().unwrap();
    let instruction = program
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|instruction| select(&instruction.kind))
        .unwrap();
    let count = clear(&mut instruction.kind);
    assert_eq!(
        before - program.retained_bytes().unwrap(),
        count * size_of::<ValueId>()
    );
}

fn clear(kind: &mut CoreInstructionKind) -> usize {
    let values = match kind {
        CoreInstructionKind::Closure {
            captures: values, ..
        }
        | CoreInstructionKind::Invoke {
            arguments: values, ..
        }
        | CoreInstructionKind::Call {
            arguments: values, ..
        }
        | CoreInstructionKind::List { elements: values }
        | CoreInstructionKind::Tuple { elements: values } => values,
        _ => unreachable!(),
    };
    let count = values.len();
    values.clear();
    count
}
