use std::sync::Arc;

use super::ProjectionContract;
use crate::program::expression::core::metadata::{
    BindingRoot, CallableContract, CoreValueMetadata, DeferredCall, DeferredUse, DependencyMask,
    MetadataPath, ProjectionStep, Stage,
};
use crate::program::expression::FunctionEffect;
use crate::program::{FieldIndex, VariantIndex};

impl CoreValueMetadata {
    pub(crate) fn list(elements: &[Self]) -> Self {
        let mut output = Self::constant();
        for element in elements {
            output.absorb_leaf(element);
        }
        if !elements.is_empty() {
            output.projection = ProjectionContract::Known(
                vec![(ProjectionStep::CollectionElement, joined(elements))].into(),
            );
        }
        output
    }

    pub(crate) fn tuple(elements: &[Self]) -> Self {
        Self::constructed(
            elements
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    (
                        ProjectionStep::TupleElement(
                            u16::try_from(index).expect("tuple arity fits u16"),
                        ),
                        value.clone(),
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn append_map_value(&mut self, value: &Self) {
        let entry = Self::tuple(&[Self::constant(), value.clone()]);
        let mut entries = match &self.projection {
            ProjectionContract::Known(fields) => fields
                .iter()
                .find(|(step, _)| *step == ProjectionStep::CollectionElement)
                .map(|(_, value)| vec![value.clone()])
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        entries.push(entry);
        self.absorb_leaf(value);
        self.projection = ProjectionContract::Known(
            vec![(ProjectionStep::CollectionElement, joined(&entries))].into(),
        );
    }

    pub(crate) fn structure(fields: &[Self]) -> Self {
        Self::constructed(
            fields
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    (
                        ProjectionStep::StructField(field_index(index)),
                        value.clone(),
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn enumeration(variant: VariantIndex, fields: &[Self]) -> Self {
        Self::constructed(
            fields
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    (
                        ProjectionStep::EnumField {
                            variant,
                            field: field_index(index),
                        },
                        value.clone(),
                    )
                })
                .collect(),
        )
    }

    fn constructed(fields: Vec<(ProjectionStep, Self)>) -> Self {
        let mut output = Self::constant();
        for (_, field) in &fields {
            output.absorb_leaf(field);
        }
        output.projection = ProjectionContract::Known(fields.into());
        output
    }

    pub(super) fn binding_projection(
        root: BindingRoot,
        path: MetadataPath,
        callable: Option<FunctionEffect>,
        enum_field: bool,
    ) -> Self {
        let mut output = Self::pure(
            Stage::Const,
            DependencyMask::binding_shape(root, path.clone()),
            DependencyMask::binding_leaf(root, path.clone()),
        );
        if enum_field {
            output
                .shape_dependencies
                .union(&DependencyMask::binding_shape(
                    root,
                    MetadataPath::default(),
                ));
        }
        output.projection = ProjectionContract::Binding {
            root,
            path: path.clone(),
        };
        output.callable = callable.map(|fallback| CallableContract::Binding {
            root,
            path,
            fallback,
        });
        output
    }

    pub(crate) fn impossible(callable: bool) -> Self {
        let mut output = Self::constant();
        output.projection = ProjectionContract::Impossible;
        output.callable = callable.then_some(CallableContract::Impossible);
        output
    }

    pub(crate) fn deferred_projection(
        call: &Arc<DeferredCall>,
        path: MetadataPath,
        callable: bool,
    ) -> Self {
        let mut output = Self::constant();
        output
            .deferred
            .push(DeferredUse::new(Arc::clone(call), path.clone()));
        output.projection = ProjectionContract::Deferred {
            call: Arc::clone(call),
            path: path.clone(),
        };
        output.callable = callable.then_some(CallableContract::Deferred {
            call: Arc::clone(call),
            path,
        });
        output
    }
}

fn joined(values: &[CoreValueMetadata]) -> CoreValueMetadata {
    let refs = values.iter().collect::<Vec<_>>();
    let mut output = CoreValueMetadata::combine(refs.iter().copied());
    output.join_contract_from(&refs);
    output
}

fn field_index(index: usize) -> FieldIndex {
    FieldIndex::from_position(index).expect("nominal field limit fits FieldIndex")
}
