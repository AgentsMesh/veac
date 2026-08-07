use std::ops::Range;

use super::super::{error, Definitions};
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, ValueId,
};
use crate::program::expression::{ExpressionError, Stage, TemporalAttachmentKind, ValueType};

pub(super) fn verify(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let CoreInstructionKind::TemporalAttach {
        kind,
        owner,
        selectors,
        animation,
    } = instruction.kind()
    else {
        unreachable!("temporal attachment verifier contract")
    };
    let kind = TemporalAttachmentKind::from_opcode(*kind)
        .ok_or_else(|| error("unknown temporal attachment kind", instruction.span()))?;
    require_type(
        program,
        definitions,
        *owner,
        &kind.owner_type(),
        instruction.span(),
    )?;
    if selectors.len() != kind.selector_types().len() {
        return Err(error(
            "temporal attachment selector arity mismatch",
            instruction.span(),
        ));
    }
    for (selector, primitive) in selectors.iter().zip(kind.selector_types()) {
        require_type(
            program,
            definitions,
            *selector,
            &ValueType::primitive(*primitive),
            instruction.span(),
        )?;
    }
    require_type(
        program,
        definitions,
        *animation,
        &kind.animation_type(),
        instruction.span(),
    )?;
    for value in std::iter::once(owner).chain(selectors).chain([animation]) {
        let metadata = definitions.metadata(*value);
        if metadata.shape_stage() == Stage::Temporal || metadata.leaf_stage() == Stage::Temporal {
            return Err(error(
                "temporal attachment topology must close at Build stage",
                instruction.span(),
            ));
        }
    }
    require_type(
        program,
        definitions,
        instruction.id(),
        &kind.owner_type(),
        instruction.span(),
    )
}

fn require_type(
    program: &CoreProgram,
    definitions: &Definitions,
    value: ValueId,
    expected: &ValueType,
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    let actual = super::value_type(program, definitions, value, span.clone())?;
    (actual == expected).then_some(()).ok_or_else(|| {
        error(
            format!("temporal attachment expects {expected}, found {actual}"),
            span,
        )
    })
}
