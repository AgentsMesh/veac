use std::sync::Arc;

use super::{error, verify_inner, VerifiedCoreProgram, VerifyBudget, VerifyContext};
use crate::program::expression::core::{
    closure_digest, ClosureDefinitionId, CoreDigest, CoreProgram, FunctionSummary,
};
use crate::program::expression::{
    Effect, ExpressionError, FunctionEffect, Stage, ValueType, MAX_CLOSURE_CAPTURES,
    MAX_FUNCTION_PARAMETERS,
};

mod stage;

#[derive(Debug, Clone)]
pub(crate) struct VerifiedClosureDefinition {
    id: ClosureDefinitionId,
    parameter_types: Arc<[ValueType]>,
    parameter_stages: Arc<[Stage]>,
    capture_types: Arc<[ValueType]>,
    non_escaping: bool,
    value_type: ValueType,
    body: VerifiedCoreProgram,
    summary: FunctionSummary,
    digest: CoreDigest,
}

impl VerifiedClosureDefinition {
    pub(crate) fn id(&self) -> ClosureDefinitionId {
        self.id
    }

    pub(crate) fn parameter_types(&self) -> &[ValueType] {
        &self.parameter_types
    }

    pub(crate) fn parameter_stages(&self) -> &[Stage] {
        &self.parameter_stages
    }

    pub(crate) fn capture_types(&self) -> &[ValueType] {
        &self.capture_types
    }

    pub(crate) fn non_escaping(&self) -> bool {
        self.non_escaping
    }

    pub(crate) fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub(crate) fn body(&self) -> &VerifiedCoreProgram {
        &self.body
    }

    pub(crate) fn summary(&self) -> &FunctionSummary {
        &self.summary
    }

    pub(crate) fn digest(&self) -> CoreDigest {
        self.digest
    }
}

pub(super) fn verify_all(
    program: &CoreProgram,
    nominal_types: &crate::program::TypeRegistry,
    depth: usize,
    budget: &mut VerifyBudget,
    context: &VerifyContext<'_>,
) -> Result<Vec<Arc<VerifiedClosureDefinition>>, ExpressionError> {
    program
        .closure_definitions
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            if definition.id.index() != Some(index) {
                return Err(error(
                    "closure definition IDs must be dense",
                    definition.span(),
                ));
            }
            if definition.parameter_types.len() > MAX_FUNCTION_PARAMETERS
                || definition.capture_types.len() > MAX_CLOSURE_CAPTURES
                || definition.parameter_stages.len() != definition.parameter_types.len()
            {
                return Err(error(
                    "closure signature exceeds its arity limit",
                    definition.span(),
                ));
            }
            if !definition.non_escaping
                && definition
                    .capture_types
                    .iter()
                    .any(|value| value.contains_function_in(nominal_types) != Some(false))
            {
                return Err(error(
                    "escaping closure cannot capture function values",
                    definition.span(),
                ));
            }
            if !definition.body.inputs.is_empty() {
                return Err(error(
                    "closure body must use explicit captures instead of external inputs",
                    definition.span(),
                ));
            }
            stage::verify(program, definition)?;
            let body = verify_inner(
                (*definition.body).clone(),
                &definition.parameter_types,
                &definition.parameter_stages,
                &definition.capture_types,
                depth + 1,
                budget,
                context,
            )?;
            let value_type = ValueType::function(
                definition.parameter_types.clone(),
                body.core().result_type().clone(),
                definition.effect,
            )
            .map_err(|_| error("closure function type is invalid", definition.span()))?;
            let summary = body.core().function_summary();
            if summary != definition.summary {
                return Err(error(
                    "closure summary does not match its verified body",
                    definition.span(),
                ));
            }
            verify_effect_contract(definition.effect, &summary, definition.span())?;
            let digest = closure_digest(
                &definition.parameter_types,
                &definition.parameter_stages,
                &definition.capture_types,
                definition.effect,
                definition.non_escaping,
                body.core(),
            );
            if digest != definition.digest {
                return Err(error(
                    "closure digest does not match its verified body",
                    definition.span(),
                ));
            }
            Ok(Arc::new(VerifiedClosureDefinition {
                id: definition.id,
                parameter_types: definition.parameter_types.clone().into(),
                parameter_stages: definition.parameter_stages.clone().into(),
                capture_types: definition.capture_types.clone().into(),
                non_escaping: definition.non_escaping,
                value_type,
                body,
                summary,
                digest,
            }))
        })
        .collect()
}

fn verify_effect_contract(
    contract: FunctionEffect,
    summary: &FunctionSummary,
    span: std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    let allowed = match contract {
        FunctionEffect::Pure => summary.effect() == Effect::Pure,
        FunctionEffect::Local => summary.effect() != Effect::GraphEmit,
        FunctionEffect::Emit => {
            summary.effect() != Effect::LocalMutation && !summary.contains_local_mutation()
        }
        FunctionEffect::Any => true,
    };
    if allowed {
        Ok(())
    } else {
        Err(error(
            format!("closure body effect does not satisfy declared effect {contract}"),
            span,
        ))
    }
}
