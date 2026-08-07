use crate::program::executable::{
    AttachedTemporalLeaf, ExecutableTemporalLeaf, ExecutableTemporalSink,
};
use crate::program::expression::{
    residualize_closure_with_ledger, residualize_expression_with_ledger, CoreTemporalInputIdentity,
    ResidualBuildBindings, ResidualLedger, ResidualizedExpression,
};
use veac_ir::{TemporalBindingId, TemporalParameterId, TemporalValue};

use super::sink::Context;
use super::{error, ExecutableLowerError};

#[derive(Clone, Copy)]
pub(super) enum Producer<'a, 'g> {
    Static(&'a ExecutableTemporalLeaf),
    Attached(&'a AttachedTemporalLeaf<'g>),
}

impl<'a> Producer<'a, '_> {
    pub(super) fn sink(self) -> &'a ExecutableTemporalSink {
        match self {
            Self::Static(value) => value.sink(),
            Self::Attached(value) => value.sink(),
        }
    }

    pub(super) fn binding_id(self) -> &'a TemporalBindingId {
        match self {
            Self::Static(value) => value.binding_id(),
            Self::Attached(value) => value.binding_id(),
        }
    }

    pub(super) fn parameter_values(
        self,
    ) -> Option<&'a std::collections::BTreeMap<TemporalParameterId, TemporalValue>> {
        match self {
            Self::Static(value) => Some(value.parameter_values()),
            Self::Attached(_) => None,
        }
    }

    pub(super) fn residual(
        self,
        build_inputs: &ResidualBuildBindings,
        context: &Context,
        ledger: ResidualLedger<'_>,
    ) -> Result<ResidualizedExpression, ExecutableLowerError> {
        match self {
            Self::Static(value) => {
                let bindings = merge_bindings(value.build_inputs(), build_inputs)?;
                residualize_expression_with_ledger(
                    value.expression(),
                    &bindings,
                    value.request().clone(),
                    ledger,
                )
                .map_err(residual_error)
            }
            Self::Attached(value) => residualize_closure_with_ledger(
                value.animation(),
                &clocks(context),
                value.request().clone(),
                ledger,
            )
            .map_err(residual_error),
        }
    }
}

fn clocks(context: &Context) -> Vec<Option<CoreTemporalInputIdentity>> {
    let sequence = &context.sequence_id;
    let sequence_time = Some(CoreTemporalInputIdentity::SequenceTime {
        sequence_id: sequence.clone(),
    });
    let frame = Some(CoreTemporalInputIdentity::Frame {
        sequence_id: sequence.clone(),
    });
    let Some(item) = &context.item_id else {
        return vec![sequence_time, frame];
    };
    vec![
        sequence_time,
        Some(CoreTemporalInputIdentity::ClipTime {
            item_id: item.clone(),
        }),
        frame,
        Some(CoreTemporalInputIdentity::Progress {
            item_id: item.clone(),
        }),
        context
            .source_material
            .clone()
            .map(|source_id| CoreTemporalInputIdentity::SourceTime { source_id }),
    ]
}

fn merge_bindings(
    local: &ResidualBuildBindings,
    global: &ResidualBuildBindings,
) -> Result<ResidualBuildBindings, ExecutableLowerError> {
    let mut bindings = local.clone();
    for (id, value) in global {
        if let Some(existing) = bindings.insert(*id, value.clone()) {
            if existing != *value {
                return Err(error(
                    "EXECUTABLE_TEMPORAL_BUILD_INPUT_COLLISION",
                    "a Temporal Build input identity has conflicting values",
                ));
            }
        }
    }
    Ok(bindings)
}

fn residual_error(value: crate::program::expression::ResidualizationError) -> ExecutableLowerError {
    error("EXECUTABLE_TEMPORAL_RESIDUAL", value.to_string())
}
