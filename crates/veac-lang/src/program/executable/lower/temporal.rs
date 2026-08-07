use crate::program::executable::{self, ExecutableTemporalLeaf};
use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::{ExecutionBudget, ResidualBuildBindings, ResidualRuntimeValue};
use veac_ir::{Project, TemporalProgramLibrary};

use super::error::ExecutableLowerError;

mod binding;
mod library;
mod producer;
mod sink;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    project: &mut Project,
    leaves: &[ExecutableTemporalLeaf],
    build_inputs: &ResidualBuildBindings,
    ledger: &ExecutionBudget,
) -> Result<TemporalProgramLibrary, ExecutableLowerError> {
    let attached = executable::compile_attached(graph)?;
    if leaves.is_empty() && attached.is_empty() {
        return Ok(super::manifest::empty_temporal());
    }
    let mut ordered = leaves
        .iter()
        .map(producer::Producer::Static)
        .chain(attached.iter().map(producer::Producer::Attached))
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.sink().cmp(right.sink()));
    for pair in ordered.windows(2) {
        if pair[0].sink() == pair[1].sink() {
            return Err(error(
                "EXECUTABLE_TEMPORAL_SINK_DUPLICATE",
                "one canonical animation leaf has more than one temporal producer",
            ));
        }
    }

    let mut library = library::Builder::default();
    let mut attachments = Vec::with_capacity(ordered.len());
    let residual_ledger = ledger.residual_ledger();
    let empty_parameters = std::collections::BTreeMap::new();
    for leaf in ordered {
        let context = sink::context(project, leaf.sink())?;
        let residual = leaf.residual(build_inputs, &context, residual_ledger)?;
        let ResidualRuntimeValue::Residual(value) = residual.value() else {
            return Err(error(
                "EXECUTABLE_TEMPORAL_CONCRETE",
                "a temporal host leaf must depend on at least one declared Temporal input",
            ));
        };
        if value.value_type() != leaf.sink().expected_type() {
            return Err(error(
                "EXECUTABLE_TEMPORAL_SINK_TYPE",
                "residual result type does not match the closed canonical animation sink",
            ));
        }
        let program = residual.program().cloned().ok_or_else(|| {
            error(
                "EXECUTABLE_TEMPORAL_PROGRAM_MISSING",
                "a residual animation leaf did not publish its Temporal program",
            )
        })?;
        let program_id = library.program(program)?;
        let parameters = leaf.parameter_values().unwrap_or(&empty_parameters);
        let lowered = binding::lower(
            leaf.sink(),
            leaf.binding_id(),
            parameters,
            &residual,
            program_id,
            &context,
        )?;
        library.provenance(residual.provenance().clone())?;
        library.binding(lowered)?;
        attachments.push((leaf.sink().clone(), leaf.binding_id().clone()));
    }
    let library = library.finish()?;
    for (target, binding_id) in attachments {
        sink::attach(project, &target, binding_id)?;
    }
    Ok(library)
}

pub(super) fn error(reason: &'static str, message: impl Into<String>) -> ExecutableLowerError {
    ExecutableLowerError::lower(reason, message)
}

#[cfg(test)]
mod tests;
