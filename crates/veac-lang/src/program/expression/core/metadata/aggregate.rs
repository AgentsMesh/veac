use super::{
    CoreValueMetadata, DeferredUse, DependencyMask, EffectEvidence, FunctionSummary,
    ProjectionContract, Stage,
};
use crate::program::expression::{CollectionOperation, ValueType};

impl CoreValueMetadata {
    pub(crate) fn aggregate(
        operation: CollectionOperation,
        iterable: &Self,
        initial: Option<&Self>,
        callback: &Self,
        callback_result: &ValueType,
        _result_type: &ValueType,
    ) -> Option<Self> {
        let invoked =
            Self::aggregate_invocation(operation, iterable, initial, callback, callback_result)?;
        let output = match operation {
            CollectionOperation::Map => map_output(iterable, callback, &invoked),
            CollectionOperation::Filter => {
                let mut output = Self::constant();
                output.absorb_shape(iterable);
                output.absorb_shape(&invoked);
                output.absorb_leaf(iterable);
                output.absorb_leaf(&invoked);
                output.absorb_effect(callback);
                output.projection = iterable.projection.clone();
                output
            }
            CollectionOperation::Fold => {
                let initial = initial?;
                let mut output = Self::combine([initial, &invoked]);
                output.absorb_shape(&axis(iterable, true));
                output.absorb_effect(iterable);
                output.absorb_effect(callback);
                output.join_contract_from(&[initial, &invoked]);
                output
            }
        };
        Some(output)
    }

    pub(crate) fn for_each(
        iterable: &Self,
        summary: &FunctionSummary,
        captures: &[Self],
        callback_result: &ValueType,
        _result_type: &ValueType,
    ) -> Option<Self> {
        let callback = Self::closure(summary, captures);
        let arguments = [element_metadata(iterable), iteration_index(iterable)];
        let invoked = Self::invoke(&callback, &arguments, callback_result)?;
        Some(map_output(iterable, &callback, &invoked))
    }

    pub(crate) fn aggregate_invocation(
        operation: CollectionOperation,
        iterable: &Self,
        initial: Option<&Self>,
        callback: &Self,
        callback_result: &ValueType,
    ) -> Option<Self> {
        let element = element_metadata(iterable);
        let arguments = match operation {
            CollectionOperation::Map | CollectionOperation::Filter => vec![element],
            CollectionOperation::Fold => vec![initial?.clone(), element],
        };
        Self::invoke(callback, &arguments, callback_result)
    }
}

fn map_output(
    iterable: &CoreValueMetadata,
    callback: &CoreValueMetadata,
    invoked: &CoreValueMetadata,
) -> CoreValueMetadata {
    let mut output = CoreValueMetadata::constant();
    output.absorb_shape(&axis(iterable, true));
    output.absorb_leaf(invoked);
    output.absorb_effect(callback);
    output.projection = CoreValueMetadata::list(std::slice::from_ref(invoked)).projection;
    output
}

fn iteration_index(iterable: &CoreValueMetadata) -> CoreValueMetadata {
    let shape = axis(iterable, true);
    CoreValueMetadata {
        effect: EffectEvidence::PURE,
        shape_stage: shape.shape_stage,
        leaf_stage: shape.shape_stage,
        shape_dependencies: shape.shape_dependencies.clone(),
        leaf_dependencies: shape.shape_dependencies,
        callable: None,
        deferred: shape.deferred,
        projection: ProjectionContract::Opaque,
    }
}

fn element_metadata(iterable: &CoreValueMetadata) -> CoreValueMetadata {
    let leaf = axis(iterable, false);
    let contract = iterable
        .collection_element()
        .unwrap_or_else(CoreValueMetadata::constant);
    CoreValueMetadata {
        effect: EffectEvidence::PURE,
        shape_stage: leaf.leaf_stage,
        leaf_stage: leaf.leaf_stage,
        shape_dependencies: leaf.leaf_dependencies.clone(),
        leaf_dependencies: leaf.leaf_dependencies,
        callable: contract.callable,
        deferred: leaf
            .deferred
            .into_iter()
            .map(|value| DeferredUse {
                call: value.call,
                path: value.path,
                shape: true,
                leaf: true,
            })
            .collect(),
        projection: contract.projection,
    }
}

fn axis(value: &CoreValueMetadata, shape: bool) -> CoreValueMetadata {
    let (stage, dependencies) = if shape {
        (value.shape_stage, value.shape_dependencies.clone())
    } else {
        (value.leaf_stage, value.leaf_dependencies.clone())
    };
    let deferred = value
        .deferred
        .iter()
        .filter(|value| if shape { value.shape } else { value.leaf })
        .map(|value| DeferredUse {
            call: std::sync::Arc::clone(&value.call),
            path: value.path.clone(),
            shape,
            leaf: !shape,
        })
        .collect();
    CoreValueMetadata {
        effect: value.effect,
        shape_stage: if shape { stage } else { Stage::Const },
        leaf_stage: if shape { Stage::Const } else { stage },
        shape_dependencies: if shape {
            dependencies.clone()
        } else {
            DependencyMask::default()
        },
        leaf_dependencies: if shape {
            DependencyMask::default()
        } else {
            dependencies
        },
        callable: None,
        deferred,
        projection: ProjectionContract::Opaque,
    }
}
