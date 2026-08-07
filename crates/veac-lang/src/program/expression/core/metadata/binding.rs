use std::sync::Arc;

use super::callable::{join, CallableContract, DeferredCall};
use super::{
    BindingRoot, CoreValueMetadata, DependencyMask, FunctionSummary, MetadataPath,
    ProjectionContract, Stage,
};
use crate::program::expression::{FunctionEffect, InputId, ValueType, ValueTypeKind};

mod axis;
use axis::bind as bind_axis;

impl CoreValueMetadata {
    pub(crate) fn temporal_input(id: InputId) -> Self {
        let input = Self::input(Stage::Temporal, id);
        let mut output = Self::constant();
        output.absorb_leaf(&input);
        output
    }

    pub(crate) fn typed_parameter(stage: Stage, index: usize, value_type: &ValueType) -> Self {
        let mut value = Self::parameter(stage, index);
        if stage == Stage::Temporal {
            value.shape_stage = Stage::Const;
            value.shape_dependencies = DependencyMask::default();
        }
        value.projection = ProjectionContract::Binding {
            root: BindingRoot::Parameter(index),
            path: MetadataPath::default(),
        };
        if let Some(fallback) = function_effect(value_type) {
            value.callable = Some(CallableContract::Binding {
                root: BindingRoot::Parameter(index),
                path: MetadataPath::default(),
                fallback,
            });
        }
        value
    }

    pub(crate) fn capture(stage: Stage, index: usize, value_type: &ValueType) -> Self {
        let mut value = Self::pure(
            stage,
            DependencyMask::capture_shape(index),
            DependencyMask::capture_leaf(index),
        );
        value.projection = ProjectionContract::Binding {
            root: BindingRoot::Capture(index),
            path: MetadataPath::default(),
        };
        if let Some(fallback) = function_effect(value_type) {
            value.callable = Some(CallableContract::Binding {
                root: BindingRoot::Capture(index),
                path: MetadataPath::default(),
                fallback,
            });
        }
        value
    }

    pub(crate) fn closure(summary: &FunctionSummary, captures: &[Self]) -> Self {
        let mut value = Self::combine(captures);
        value.callable = Some(CallableContract::Closure {
            summary: Arc::new(summary.clone()),
            captures: captures.to_vec().into(),
        });
        value
    }
    pub(crate) fn invoke(callee: &Self, arguments: &[Self], result: &ValueType) -> Option<Self> {
        Self::invoke_contract(callee.callable.clone()?, arguments, result)
    }

    pub(super) fn invoke_contract(
        callee: CallableContract,
        arguments: &[Self],
        result: &ValueType,
    ) -> Option<Self> {
        match callee {
            CallableContract::Closure { summary, captures } => {
                summary.instantiate(arguments, &captures)
            }
            CallableContract::Join(values) => {
                let results = values
                    .iter()
                    .map(|value| Self::invoke_contract(value.clone(), arguments, result));
                Self::join_function_results(results, result)
            }
            CallableContract::Impossible => Some(Self::impossible(matches!(
                result.kind(),
                ValueTypeKind::Function { .. }
            ))),
            callee => Some(deferred(callee, arguments, result)),
        }
    }
    pub(crate) fn join_function(values: &[&Self]) -> Option<CallableContract> {
        join(values.iter().filter_map(|value| value.callable.clone()))
    }
    pub(crate) fn bind(&self, parameters: &[Self], captures: &[Self]) -> Option<Self> {
        let (shape_stage, shape_dependencies) = bind_axis(
            self.shape_stage,
            &self.shape_dependencies,
            parameters,
            captures,
        )?;
        let (leaf_stage, leaf_dependencies) = bind_axis(
            self.leaf_stage,
            &self.leaf_dependencies,
            parameters,
            captures,
        )?;
        let mut output = Self {
            effect: self.effect,
            shape_stage,
            leaf_stage,
            shape_dependencies,
            leaf_dependencies,
            callable: match &self.callable {
                Some(value) => Some(value.bind(parameters, captures)?),
                None => None,
            },
            deferred: Vec::new(),
            projection: self.projection.bind(parameters, captures)?,
        };
        for value in &self.deferred {
            let resolved = value
                .call
                .bind(parameters, captures)?
                .project_path(&value.path, false)?;
            output.effect = output.effect.join(resolved.effect);
            if value.shape {
                output.absorb_shape(&resolved);
            }
            if value.leaf {
                output.absorb_leaf(&resolved);
            }
            if !value.shape && !value.leaf {
                merge_effect_only(&mut output, &resolved);
            }
        }
        Some(output)
    }

    pub(crate) fn absorb_effect(&mut self, value: &Self) {
        merge_effect_only(self, value);
    }

    fn join_function_results(
        values: impl Iterator<Item = Option<Self>>,
        result: &ValueType,
    ) -> Option<Self> {
        let values = values.collect::<Option<Vec<_>>>()?;
        let refs = values.iter().collect::<Vec<_>>();
        let mut output = Self::combine(refs.iter().copied());
        output.join_contract_from(&refs);
        if matches!(result.kind(), ValueTypeKind::Function { .. }) {
            output.callable = Self::join_function(&refs);
        }
        Some(output)
    }
}

fn function_effect(value_type: &ValueType) -> Option<FunctionEffect> {
    match value_type.kind() {
        ValueTypeKind::Function { effect, .. } => Some(effect),
        _ => None,
    }
}

fn deferred(
    callee: CallableContract,
    arguments: &[CoreValueMetadata],
    result: &ValueType,
) -> CoreValueMetadata {
    let call = Arc::new(DeferredCall {
        callee,
        arguments: arguments.to_vec().into(),
        result_type: result.clone(),
    });
    let mut output = CoreValueMetadata::deferred_projection(
        &call,
        MetadataPath::default(),
        matches!(result.kind(), ValueTypeKind::Function { .. }),
    );
    output.deferred[0].shape = true;
    output.deferred[0].leaf = true;
    output
}

fn merge_effect_only(output: &mut CoreValueMetadata, value: &CoreValueMetadata) {
    output.effect = output.effect.join(value.effect);
    for deferred in &value.deferred {
        let deferred = deferred.with_axes(false, false);
        if !output.deferred.contains(&deferred) {
            output.deferred.push(deferred);
        }
    }
}
