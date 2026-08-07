use std::ops::Range;

use super::{CoreProgram, FunctionRegistry, CORE_VERSION};
use crate::program::expression::{ExpressionError, FunctionParameter, Stage, ValueType};

mod aggregate;
mod budget;
mod closure;
mod collection;
mod context;
mod control;
mod definitions;
mod domain;
mod effect;
mod for_each;
mod input;
mod instruction;
mod local;
mod map_protocol;
mod metadata;
mod nominal;
mod nominal_value;
mod non_escaping;
mod range;
mod temporal;
mod terminator;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod type_table;
mod types;

pub(crate) use closure::VerifiedClosureDefinition;

use budget::VerifyBudget;
use context::VerifyContext;
use control::ControlFlow;
use definitions::Definitions;

#[derive(Debug, Clone)]
pub(crate) struct VerifiedCoreProgram {
    core: CoreProgram,
    closures: Vec<std::sync::Arc<closure::VerifiedClosureDefinition>>,
    nominal_types: std::sync::Arc<crate::program::TypeRegistry>,
}

impl VerifiedCoreProgram {
    pub(crate) fn core(&self) -> &CoreProgram {
        &self.core
    }

    pub(crate) fn closure(
        &self,
        id: super::ClosureDefinitionId,
    ) -> Option<&std::sync::Arc<closure::VerifiedClosureDefinition>> {
        id.index().and_then(|index| self.closures.get(index))
    }

    pub(crate) fn closures(&self) -> &[std::sync::Arc<closure::VerifiedClosureDefinition>] {
        &self.closures
    }

    pub(crate) fn nominal_types(&self) -> &crate::program::TypeRegistry {
        &self.nominal_types
    }
}

pub(crate) fn verify(
    program: CoreProgram,
    registry: &FunctionRegistry,
    parameters: &[FunctionParameter],
) -> Result<VerifiedCoreProgram, ExpressionError> {
    verify_with_input_trust(program, registry, parameters, &|_| false)
}

pub(crate) fn verify_with_input_trust(
    program: CoreProgram,
    registry: &FunctionRegistry,
    parameters: &[FunctionParameter],
    trusted_functions: &dyn Fn(&str) -> bool,
) -> Result<VerifiedCoreProgram, ExpressionError> {
    let domain_registry = crate::program::DomainOperationRegistry::shared();
    let parameters = parameters
        .iter()
        .map(|parameter| parameter.value_type.clone())
        .collect::<Vec<_>>();
    let parameter_stages = vec![Stage::Const; parameters.len()];
    let mut budget = VerifyBudget::default();
    let context = VerifyContext {
        functions: registry,
        input_trust: trusted_functions,
        domain: domain_registry,
    };
    verify_inner(
        program,
        &parameters,
        &parameter_stages,
        &[],
        0,
        &mut budget,
        &context,
    )
}

fn verify_inner(
    program: CoreProgram,
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
    depth: usize,
    budget: &mut VerifyBudget,
    context: &VerifyContext<'_>,
) -> Result<VerifiedCoreProgram, ExpressionError> {
    if program.version != CORE_VERSION {
        return Err(error(
            format!("unsupported Core version {}", program.version),
            0..0,
        ));
    }
    domain::identity(&program, context.domain)?;
    domain::called_identities(&program, context.functions)?;
    budget.enter(&program, depth)?;
    let nominal_types = std::sync::Arc::new(nominal::verify(&program, parameters, captures)?);
    type_table::verify(&program)?;
    input::verify_all(&program, &nominal_types, context.input_trust)?;
    let definitions = Definitions::collect(&program)?;
    let control = ControlFlow::analyze(&program)?;
    local::verify(
        &program,
        &definitions,
        &control,
        &nominal_types,
        parameters,
        parameter_stages,
        captures,
    )?;
    let closures = closure::verify_all(&program, &nominal_types, depth, budget, context)?;
    for (block_index, block) in program.blocks.iter().enumerate() {
        for (position, value) in block.instructions.iter().enumerate() {
            instruction::verify(
                value,
                block_index,
                position,
                &program,
                &definitions,
                &control,
                context.functions,
                context.domain,
                &nominal_types,
                parameters,
                parameter_stages,
                captures,
            )?;
        }
    }
    map_protocol::verify(&program)?;
    for (block_index, block) in program.blocks.iter().enumerate() {
        terminator::verify(
            &block.terminator,
            block_index,
            block.instructions.len(),
            &program,
            &definitions,
            &control,
            &nominal_types,
            &closures,
        )?;
    }
    non_escaping::verify(&program, &closures)?;
    metadata::block_parameters(&program, &definitions, &control, &nominal_types)?;
    definitions.verify_depth()?;
    Ok(VerifiedCoreProgram {
        core: program,
        closures,
        nominal_types,
    })
}

pub(super) fn error(message: impl Into<String>, span: Range<usize>) -> ExpressionError {
    ExpressionError::new("EXPRESSION_CORE_VERIFY", message, span)
}
