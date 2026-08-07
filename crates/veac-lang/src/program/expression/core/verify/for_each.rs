use super::closure::VerifiedClosureDefinition;
use super::control::ControlFlow;
use super::definitions::Definitions;
use super::{error, metadata};
use crate::program::expression::core::{
    CoreForEach, CoreForEachOrder, CoreForEachSlotId, CoreProgram, CoreValueMetadata,
};
use crate::program::expression::{
    ExpressionError, FunctionEffect, MapKeyType, PrimitiveType, Stage, ValueType, ValueTypeKind,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    value: &CoreForEach,
    block: usize,
    position: usize,
    program: &CoreProgram,
    definitions: &Definitions,
    control: &ControlFlow,
    closures: &[std::sync::Arc<VerifiedClosureDefinition>],
) -> Result<(), ExpressionError> {
    let span = value.provenance.loop_span.clone();
    verify_provenance(value)?;
    verify_bound_and_order(value)?;
    definitions.verify_use(value.iterable, block, position, control, span.clone())?;
    let iterable_type = program
        .value_type(definitions.type_id(value.iterable))
        .ok_or_else(|| error("for-each iterable must have a value type", span.clone()))?;
    let element_type = iterable_element(iterable_type, &span)?;
    if definitions.metadata(value.iterable).shape_stage() > Stage::Build {
        return Err(error(
            "for-each shape cannot depend on Temporal-stage data",
            span,
        ));
    }
    let definition = value
        .body
        .index()
        .and_then(|index| closures.get(index))
        .filter(|definition| definition.id() == value.body)
        .ok_or_else(|| error("for-each body definition is unavailable", span.clone()))?;
    verify_body(
        value,
        definition,
        &element_type,
        program,
        definitions,
        block,
        position,
        control,
    )?;
    let continuation = value
        .continuation
        .index()
        .and_then(|index| program.blocks.get(index))
        .ok_or_else(|| error("for-each continuation is unavailable", span.clone()))?;
    if !control.exclusive_predecessor(
        value.continuation.index().expect("verified continuation"),
        block,
    ) {
        return Err(error(
            "for-each continuation must have one exclusive predecessor",
            span,
        ));
    }
    let [parameter] = continuation.parameters.as_slice() else {
        return Err(error(
            "for-each continuation requires exactly one result parameter",
            span,
        ));
    };
    let result_type = ValueType::list(definition.body().core().result_type().clone())
        .map_err(|_| error("for-each result list type is invalid", span.clone()))?;
    if program.value_type(parameter.type_id) != Some(&result_type) {
        return Err(error("for-each continuation result type is invalid", span));
    }
    let captures = value
        .captures
        .iter()
        .map(|id| definitions.metadata(*id).clone())
        .collect::<Vec<_>>();
    let expected = CoreValueMetadata::for_each(
        definitions.metadata(value.iterable),
        definition.summary(),
        &captures,
        definition.body().core().result_type(),
        &result_type,
    )
    .ok_or_else(|| {
        error(
            "for-each metadata cannot be instantiated",
            value.provenance.loop_span.clone(),
        )
    })?;
    if value.result_metadata != expected
        || parameter.metadata != expected
        || value.effect != crate::program::expression::CoreForEachEffect::from_metadata(&expected)
    {
        return Err(error(
            "for-each result or effect evidence does not match its verified body",
            value.provenance.loop_span.clone(),
        ));
    }
    metadata::block_argument(
        &expected,
        &parameter.metadata,
        value.provenance.loop_span.clone(),
    )
}

fn verify_provenance(value: &CoreForEach) -> Result<(), ExpressionError> {
    let provenance = &value.provenance;
    let binding = &provenance.binding_span;
    if provenance.definition != value.body
        || binding.start >= binding.end
        || binding.start < provenance.loop_span.start
        || binding.end > provenance.loop_span.end
    {
        return Err(error(
            "for-each provenance must bind its body and a contained lexical binding span",
            provenance.loop_span.clone(),
        ));
    }
    Ok(())
}

fn verify_bound_and_order(value: &CoreForEach) -> Result<(), ExpressionError> {
    let limit = crate::program::expression::execution_budget::MAX_EXECUTION_ITERATIONS;
    if value.maximum_count == 0 || value.maximum_count as usize > limit {
        return Err(error(
            "for-each maximum count exceeds the execution limit",
            value.provenance.loop_span.clone(),
        ));
    }
    match value.order {
        CoreForEachOrder::Source => Ok(()),
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_body(
    value: &CoreForEach,
    definition: &VerifiedClosureDefinition,
    element_type: &ValueType,
    program: &CoreProgram,
    definitions: &Definitions,
    block: usize,
    position: usize,
    control: &ControlFlow,
) -> Result<(), ExpressionError> {
    let span = value.provenance.loop_span.clone();
    let index_type = ValueType::primitive(PrimitiveType::Integer);
    let expected = [element_type.clone(), index_type.clone()];
    if !definition.non_escaping()
        || definition.parameter_types() != expected
        || value.element.id != CoreForEachSlotId::Element
        || value.index.id != CoreForEachSlotId::Index
        || program.value_type(value.element.type_id) != Some(element_type)
        || program.value_type(value.index.type_id) != Some(&index_type)
    {
        return Err(error("for-each activation slots are invalid", span));
    }
    let raw = &program.closure_definitions[value.body.index().expect("verified definition")];
    if raw.effect != FunctionEffect::Emit || raw.span != value.provenance.loop_span {
        return Err(error(
            "for-each body must be a synthesized effect-checked definition",
            value.provenance.loop_span.clone(),
        ));
    }
    if value.captures.len() != definition.capture_types().len() {
        return Err(error("for-each capture arity is invalid", span));
    }
    for (capture, expected) in value.captures.iter().zip(definition.capture_types()) {
        definitions.verify_use(*capture, block, position, control, span.clone())?;
        if program.value_type(definitions.type_id(*capture)) != Some(expected) {
            return Err(error("for-each capture type is invalid", span));
        }
    }
    Ok(())
}

fn iterable_element(
    value: &ValueType,
    span: &std::ops::Range<usize>,
) -> Result<ValueType, ExpressionError> {
    match value.kind() {
        ValueTypeKind::List(value) | ValueTypeKind::Range(value) => Ok(value.clone()),
        ValueTypeKind::Map { key, value } => ValueType::tuple(vec![
            ValueType::primitive(match key {
                MapKeyType::Text => PrimitiveType::Text,
                MapKeyType::Identifier => PrimitiveType::Identifier,
            }),
            value.clone(),
        ])
        .map_err(|_| error("for-each map element type is invalid", span.clone())),
        _ => Err(error("for-each input is not iterable", span.clone())),
    }
}
