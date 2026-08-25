use super::{InputId, ValueId};

mod aggregate;
mod binding;
mod callable;
mod callable_effect;
mod dependency;
mod domain;
mod effect;
mod path;
mod projection;
mod summary;
mod temporal;
mod temporal_attachment;
pub(crate) use callable::{CallableContract, DeferredCall, DeferredUse};
pub use dependency::DependencyMask;
pub(crate) use effect::EffectEvidence;
pub(crate) use path::{BindingRoot, MetadataPath, ProjectionStep};
pub(crate) use projection::ProjectionContract;
pub use summary::FunctionSummary;
pub use veac_lang_model::{Effect, Stage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreValueMetadata {
    pub(crate) effect: EffectEvidence,
    pub(crate) shape_stage: Stage,
    pub(crate) leaf_stage: Stage,
    pub(crate) shape_dependencies: DependencyMask,
    pub(crate) leaf_dependencies: DependencyMask,
    pub(crate) callable: Option<CallableContract>,
    pub(crate) deferred: Vec<DeferredUse>,
    pub(crate) projection: ProjectionContract,
}

impl CoreValueMetadata {
    pub fn effect(&self) -> Effect {
        self.effect.summary()
    }

    pub fn contains_local_mutation(&self) -> bool {
        self.effect.contains_local_mutation()
    }

    pub fn shape_stage(&self) -> Stage {
        self.shape_stage
    }

    pub fn leaf_stage(&self) -> Stage {
        self.leaf_stage
    }

    pub fn shape_dependencies(&self) -> &DependencyMask {
        &self.shape_dependencies
    }

    pub fn leaf_dependencies(&self) -> &DependencyMask {
        &self.leaf_dependencies
    }

    pub(crate) fn constant() -> Self {
        Self::pure(
            Stage::Const,
            DependencyMask::default(),
            DependencyMask::default(),
        )
    }

    pub(crate) fn input(stage: Stage, id: InputId) -> Self {
        Self::pure(
            stage,
            DependencyMask::input_shape(id),
            DependencyMask::input_leaf(id),
        )
    }

    pub(crate) fn parameter(stage: Stage, index: usize) -> Self {
        Self::pure(
            stage,
            DependencyMask::parameter_shape(index),
            DependencyMask::parameter_leaf(index),
        )
    }

    pub(crate) fn pure(
        stage: Stage,
        shape_dependencies: DependencyMask,
        leaf_dependencies: DependencyMask,
    ) -> Self {
        Self {
            effect: EffectEvidence::PURE,
            shape_stage: stage,
            leaf_stage: stage,
            shape_dependencies,
            leaf_dependencies,
            callable: None,
            deferred: Vec::new(),
            projection: ProjectionContract::Opaque,
        }
    }

    pub(crate) fn combine<'a>(values: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut output = Self::constant();
        for value in values {
            output.effect = output.effect.join(value.effect);
            output.shape_stage = output.shape_stage.join(value.shape_stage);
            output.leaf_stage = output.leaf_stage.join(value.leaf_stage);
            output.shape_dependencies.union(&value.shape_dependencies);
            output.leaf_dependencies.union(&value.leaf_dependencies);
            merge_deferred(&mut output.deferred, &value.deferred);
        }
        output
    }

    pub(crate) fn absorb_shape(&mut self, value: &Self) {
        self.effect = self.effect.join(value.effect);
        self.shape_stage = self
            .shape_stage
            .join(value.shape_stage)
            .join(value.leaf_stage);
        self.shape_dependencies.union(&value.shape_dependencies);
        self.shape_dependencies.union(&value.leaf_dependencies);
        merge_deferred_as(&mut self.deferred, &value.deferred, true, false);
    }

    pub(crate) fn absorb_leaf(&mut self, value: &Self) {
        self.effect = self.effect.join(value.effect);
        self.leaf_stage = self
            .leaf_stage
            .join(value.shape_stage)
            .join(value.leaf_stage);
        self.leaf_dependencies.union(&value.shape_dependencies);
        self.leaf_dependencies.union(&value.leaf_dependencies);
        merge_deferred_as(&mut self.deferred, &value.deferred, false, true);
    }

    pub(crate) fn range(start: &Self, end: &Self, step: Option<&Self>) -> Self {
        let mut output = Self::constant();
        output.absorb_shape(start);
        output.absorb_shape(end);
        output.absorb_leaf(start);
        if let Some(step) = step {
            output.absorb_shape(step);
            output.absorb_leaf(step);
        }
        output
    }
}

fn merge_deferred(target: &mut Vec<DeferredUse>, values: &[DeferredUse]) {
    for value in values {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}

fn merge_deferred_as(
    target: &mut Vec<DeferredUse>,
    values: &[DeferredUse],
    shape: bool,
    leaf: bool,
) {
    for value in values {
        let value = value.with_axes(shape, leaf);
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

pub(crate) fn operand_metadata(
    operands: impl IntoIterator<Item = ValueId>,
    values: &std::collections::BTreeMap<ValueId, CoreValueMetadata>,
) -> CoreValueMetadata {
    CoreValueMetadata::combine(
        operands
            .into_iter()
            .map(|id| values.get(&id).expect("Core operand metadata exists")),
    )
}

#[cfg(test)]
#[path = "metadata/tests.rs"]
mod tests;
