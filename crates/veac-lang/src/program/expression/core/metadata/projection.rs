use std::sync::Arc;

use super::{BindingRoot, CoreValueMetadata, DeferredCall, MetadataPath, ProjectionStep};
use crate::program::expression::{FunctionEffect, ValueType, ValueTypeKind};
use crate::program::{FieldIndex, VariantIndex};

mod construct;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum ProjectionContract {
    #[default]
    Opaque,
    Binding {
        root: BindingRoot,
        path: MetadataPath,
    },
    Known(Arc<[(ProjectionStep, CoreValueMetadata)]>),
    Alternatives(Arc<[CoreValueMetadata]>),
    Deferred {
        call: Arc<DeferredCall>,
        path: MetadataPath,
    },
    Impossible,
}

impl CoreValueMetadata {
    pub(crate) fn struct_field(&self, field: FieldIndex, result: &ValueType) -> Option<Self> {
        self.project_step(ProjectionStep::StructField(field), result)
    }

    pub(crate) fn collection_element(&self) -> Option<Self> {
        self.project_step_known(ProjectionStep::CollectionElement, false, None)
    }

    pub(crate) fn enum_field(
        &self,
        variant: VariantIndex,
        field: FieldIndex,
        result: &ValueType,
    ) -> Option<Self> {
        self.project_step(ProjectionStep::EnumField { variant, field }, result)
    }

    pub(crate) fn join_contract_from(&mut self, values: &[&Self]) {
        self.callable = Self::join_function(values);
        self.projection =
            ProjectionContract::Alternatives(values.iter().map(|value| (*value).clone()).collect());
    }

    pub(crate) fn project_path(&self, path: &MetadataPath, callable: bool) -> Option<Self> {
        if let ProjectionContract::Binding { root, path: base } = &self.projection {
            let path = path
                .steps()
                .iter()
                .fold(base.clone(), |path, step| path.appended(*step));
            return Some(Self::binding_projection(*root, path, None, false));
        }
        let mut output = self.clone();
        for (index, step) in path.steps().iter().enumerate() {
            let needs_callable = callable && index + 1 == path.len();
            output = output.project_step_known(*step, needs_callable, None)?;
        }
        Some(output)
    }

    fn project_step(&self, step: ProjectionStep, result: &ValueType) -> Option<Self> {
        let fallback = function_effect(result);
        self.project_step_known(step, fallback.is_some(), fallback)
    }

    fn project_step_known(
        &self,
        step: ProjectionStep,
        callable: bool,
        fallback: Option<FunctionEffect>,
    ) -> Option<Self> {
        match &self.projection {
            ProjectionContract::Binding { root, path } => Some(Self::binding_projection(
                *root,
                path.appended(step),
                fallback,
                matches!(step, ProjectionStep::EnumField { .. }),
            )),
            ProjectionContract::Known(fields) => fields
                .iter()
                .find(|(candidate, _)| *candidate == step)
                .map(|(_, value)| value.clone())
                .or_else(|| enum_absent(step, callable)),
            ProjectionContract::Alternatives(values) => {
                let selected = values
                    .iter()
                    .map(|value| value.project_step_known(step, callable, fallback))
                    .collect::<Option<Vec<_>>>()?;
                Some(Self::joined_projection(self, &selected))
            }
            ProjectionContract::Deferred { call, path } => Some(Self::deferred_projection(
                call,
                path.appended(step),
                callable,
            )),
            ProjectionContract::Impossible => Some(Self::impossible(callable)),
            ProjectionContract::Opaque if callable => None,
            ProjectionContract::Opaque => Some(Self::opaque_projection(self)),
        }
    }

    fn joined_projection(parent: &Self, values: &[Self]) -> Self {
        let refs = values.iter().collect::<Vec<_>>();
        let mut output = Self::combine(refs.iter().copied());
        output.join_contract_from(&refs);
        output.absorb_selector(parent);
        output
    }

    fn absorb_selector(&mut self, selector: &Self) {
        self.shape_stage = self.shape_stage.join(selector.shape_stage);
        self.leaf_stage = self.leaf_stage.join(selector.shape_stage);
        self.shape_dependencies.union(&selector.shape_dependencies);
        self.leaf_dependencies.union(&selector.shape_dependencies);
    }

    fn opaque_projection(parent: &Self) -> Self {
        let mut output = Self::constant();
        output.absorb_shape(parent);
        output.absorb_leaf(parent);
        output
    }
}

fn function_effect(value_type: &ValueType) -> Option<FunctionEffect> {
    match value_type.kind() {
        ValueTypeKind::Function { effect, .. } => Some(effect),
        _ => None,
    }
}

impl ProjectionContract {
    pub(super) fn bind(
        &self,
        parameters: &[CoreValueMetadata],
        captures: &[CoreValueMetadata],
    ) -> Option<Self> {
        match self {
            Self::Opaque => Some(Self::Opaque),
            Self::Impossible => Some(Self::Impossible),
            Self::Binding { root, path } => binding(*root, parameters, captures)?
                .project_path(path, false)
                .map(|value| value.projection),
            Self::Known(fields) => Some(Self::Known(
                fields
                    .iter()
                    .map(|(step, value)| {
                        value.bind(parameters, captures).map(|value| (*step, value))
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into(),
            )),
            Self::Alternatives(values) => Some(Self::Alternatives(
                values
                    .iter()
                    .map(|value| value.bind(parameters, captures))
                    .collect::<Option<Vec<_>>>()?
                    .into(),
            )),
            Self::Deferred { call, path } => call
                .bind(parameters, captures)?
                .project_path(path, false)
                .map(|value| value.projection),
        }
    }
}

fn enum_absent(step: ProjectionStep, callable: bool) -> Option<CoreValueMetadata> {
    matches!(step, ProjectionStep::EnumField { .. })
        .then(|| CoreValueMetadata::impossible(callable))
}

fn binding<'a>(
    root: BindingRoot,
    parameters: &'a [CoreValueMetadata],
    captures: &'a [CoreValueMetadata],
) -> Option<&'a CoreValueMetadata> {
    match root {
        BindingRoot::Parameter(index) => parameters.get(index),
        BindingRoot::Capture(index) => captures.get(index),
    }
}
