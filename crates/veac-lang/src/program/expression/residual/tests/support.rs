use std::collections::BTreeMap;

use super::super::super::{
    compile_temporal_expression, CompiledExpression, CoreBuildInputId, CoreTemporalInputIdentity,
    ExpressionContext, ExpressionError, ResidualBuildBindings, ResidualizationLimits,
    ResidualizationRequest, ResidualizedExpression, TypeEnvironment, Value,
};
use veac_ir::{
    evaluate_temporal_program, TemporalAuthoredSite, TemporalDefinitionId, TemporalDefinitionKind,
    TemporalDefinitionSite, TemporalEvaluationInput, TemporalEvaluationLimits, TemporalLogicalKey,
    TemporalParameterId, TemporalProgramId, TemporalProvenance, TemporalProvenanceId,
    TemporalSourceId, TemporalSourceSpan, TemporalType, TemporalValue,
};

pub(super) fn parameter(name: &str, value_type: TemporalType) -> CoreTemporalInputIdentity {
    CoreTemporalInputIdentity::Parameter {
        parameter_id: TemporalParameterId::new(format!("tpm_{name}")).unwrap(),
        value_type,
    }
}

pub(super) fn compile(
    source: &str,
    inputs: &[(&str, CoreTemporalInputIdentity)],
) -> CompiledExpression {
    let inputs = inputs
        .iter()
        .map(|(name, identity)| ((*name).to_owned(), identity.clone()))
        .collect::<BTreeMap<_, _>>();
    compile_temporal_expression(
        source,
        &TypeEnvironment::new(),
        &inputs,
        &ExpressionContext::empty(),
    )
    .unwrap()
}

pub(super) fn compile_error(
    source: &str,
    inputs: &[(&str, CoreTemporalInputIdentity)],
) -> ExpressionError {
    let inputs = inputs
        .iter()
        .map(|(name, identity)| ((*name).to_owned(), identity.clone()))
        .collect::<BTreeMap<_, _>>();
    compile_temporal_expression(
        source,
        &TypeEnvironment::new(),
        &inputs,
        &ExpressionContext::empty(),
    )
    .unwrap_err()
}

pub(super) fn compile_with_build(
    source: &str,
    build: TypeEnvironment,
    inputs: &[(&str, CoreTemporalInputIdentity)],
) -> CompiledExpression {
    let inputs = inputs
        .iter()
        .map(|(name, identity)| ((*name).to_owned(), identity.clone()))
        .collect::<BTreeMap<_, _>>();
    compile_temporal_expression(source, &build, &inputs, &ExpressionContext::empty()).unwrap()
}

pub(super) fn request(name: &str) -> ResidualizationRequest {
    ResidualizationRequest {
        program_id: TemporalProgramId::new(format!("tpg_{name}")).unwrap(),
        provenance: provenance(name),
        limits: ResidualizationLimits::default(),
    }
}

pub(super) fn provenance(name: &str) -> TemporalProvenance {
    let definition_id = TemporalDefinitionId::new(format!("def_{name}")).unwrap();
    let source_id = TemporalSourceId::new(format!("src_{name}")).unwrap();
    let span = TemporalSourceSpan { start: 0, end: 8 };
    TemporalProvenance {
        id: TemporalProvenanceId::new(format!("tpv_{name}")).unwrap(),
        definition: TemporalDefinitionSite {
            id: definition_id.clone(),
            kind: TemporalDefinitionKind::Function,
            name: name.to_owned(),
            source_id: source_id.clone(),
            span,
        },
        origin: TemporalAuthoredSite {
            definition_id,
            function: name.to_owned(),
            source_id,
            span,
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new(format!("key_{name}")).unwrap()],
    }
}

pub(super) fn residualize(
    expression: &CompiledExpression,
    bindings: &ResidualBuildBindings,
    name: &str,
) -> ResidualizedExpression {
    super::super::residualize_expression(expression, bindings, request(name)).unwrap()
}

pub(super) fn evaluate(
    result: &ResidualizedExpression,
    inputs: Vec<(u32, TemporalValue)>,
) -> TemporalValue {
    let inputs = inputs
        .into_iter()
        .map(|(id, value)| TemporalEvaluationInput {
            input_id: veac_ir::TemporalInputId::new(id),
            value,
        })
        .collect::<Vec<_>>();
    evaluate_temporal_program(
        result.program().unwrap(),
        &inputs,
        TemporalEvaluationLimits::default(),
    )
    .unwrap()
}

pub(super) fn binding(name: &str, value: Value) -> ResidualBuildBindings {
    BTreeMap::from([(CoreBuildInputId::for_symbol(name), value)])
}

pub(super) fn no_bindings() -> ResidualBuildBindings {
    BTreeMap::new()
}
