use super::control::ControlFlow;
use super::definitions::Definitions;
use super::{error, metadata, type_table};
use crate::program::expression::core::{CoreProgram, CoreTerminator};
use crate::program::expression::{ExpressionError, PrimitiveType, Stage, ValueTypeKind};
use crate::program::{TypeDefinitionKind, TypeRegistry};

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    terminator: &CoreTerminator,
    block: usize,
    position: usize,
    program: &CoreProgram,
    definitions: &Definitions,
    control: &ControlFlow,
    registry: &TypeRegistry,
    closures: &[std::sync::Arc<super::VerifiedClosureDefinition>],
) -> Result<(), ExpressionError> {
    match terminator {
        CoreTerminator::Return { value, span } => {
            definitions.verify_use(*value, block, position, control, span.clone())?;
            require_type(
                definitions.type_id(*value),
                program.result_type,
                program,
                span,
            )
        }
        CoreTerminator::Jump {
            target,
            arguments,
            span,
        } => {
            let target = &program.blocks[target.index().expect("verified block target")];
            if arguments.len() != target.parameters.len() {
                return Err(error(
                    "jump argument count does not match target",
                    span.clone(),
                ));
            }
            for (argument, parameter) in arguments.iter().zip(&target.parameters) {
                definitions.verify_use(*argument, block, position, control, span.clone())?;
                require_type(
                    definitions.type_id(*argument),
                    parameter.type_id,
                    program,
                    span,
                )?;
                metadata::block_argument(
                    definitions.metadata(*argument),
                    &parameter.metadata,
                    span.clone(),
                )?;
            }
            Ok(())
        }
        CoreTerminator::Branch {
            condition, span, ..
        } => {
            definitions.verify_use(*condition, block, position, control, span.clone())?;
            let actual = type_table::value(program, definitions.type_id(*condition), span.clone())?;
            if actual.as_primitive() == Some(PrimitiveType::Boolean) {
                Ok(())
            } else {
                Err(error(
                    format!("expected bool, found {actual}"),
                    span.clone(),
                ))
            }
        }
        CoreTerminator::Match {
            scrutinee,
            arms,
            span,
        } => verify_match(
            *scrutinee,
            arms,
            block,
            position,
            program,
            definitions,
            control,
            registry,
            span,
        ),
        CoreTerminator::ForEach(value) => super::for_each::verify(
            value,
            block,
            position,
            program,
            definitions,
            control,
            closures,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_match(
    scrutinee: crate::program::expression::ValueId,
    arms: &[crate::program::expression::CoreMatchArm],
    block: usize,
    position: usize,
    program: &CoreProgram,
    definitions: &Definitions,
    control: &ControlFlow,
    registry: &TypeRegistry,
    span: &std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    definitions.verify_use(scrutinee, block, position, control, span.clone())?;
    let value_type = type_table::value(program, definitions.type_id(scrutinee), span.clone())?;
    let ValueTypeKind::Nominal(reference) = value_type.kind() else {
        return Err(error(
            "match scrutinee must be a nominal enum",
            span.clone(),
        ));
    };
    let Some(definition) = registry.definition(reference.id()) else {
        return Err(error("match scrutinee has an unknown TypeId", span.clone()));
    };
    let TypeDefinitionKind::Enum(layout) = definition.kind() else {
        return Err(error("match scrutinee must be an enum", span.clone()));
    };
    if definitions.metadata(scrutinee).shape_stage() > Stage::Build {
        return Err(error(
            "match selection cannot depend on Temporal-stage data",
            span.clone(),
        ));
    }
    if arms.len() != layout.variants().len() {
        return Err(error(
            "match must contain exactly one arm for every enum variant",
            span.clone(),
        ));
    }
    for (index, (arm, variant)) in arms.iter().zip(layout.variants()).enumerate() {
        if arm.variant() != variant.index() {
            return Err(error(
                "match arms must be exhaustive and ordered by variant index",
                span.clone(),
            ));
        }
        let Some(target_index) = arm.target().index() else {
            return Err(error("match arm targets an unknown block", span.clone()));
        };
        if target_index >= program.blocks.len() {
            return Err(error("match arm targets an unknown block", span.clone()));
        }
        if !control.exclusive_predecessor(target_index, block) {
            return Err(error(
                "match arm target must have exactly one exclusive predecessor",
                span.clone(),
            ));
        }
        let parameters = &program.blocks[target_index].parameters;
        if parameters.len() != variant.fields().len() {
            return Err(error(
                format!("match arm {index} payload arity does not match its variant"),
                span.clone(),
            ));
        }
        for (parameter, field) in parameters.iter().zip(variant.fields()) {
            let actual = type_table::value(program, parameter.type_id, parameter.span.clone())?;
            if actual != field.value_type() {
                return Err(error(
                    format!(
                        "match payload field `{}` expects {}, found {actual}",
                        field.name(),
                        field.value_type()
                    ),
                    parameter.span.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn require_type(
    actual: crate::program::expression::CoreTypeId,
    expected: crate::program::expression::CoreTypeId,
    program: &CoreProgram,
    span: &std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    if actual == expected {
        return Ok(());
    }
    let actual = type_table::value(program, actual, span.clone())?;
    let expected = type_table::value(program, expected, span.clone())?;
    Err(error(
        format!("expected {expected}, found {actual}"),
        span.clone(),
    ))
}
