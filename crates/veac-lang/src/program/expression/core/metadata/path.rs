use std::sync::Arc;

use crate::program::{FieldIndex, VariantIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BindingRoot {
    Parameter(usize),
    Capture(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ProjectionStep {
    CollectionElement,
    TupleElement(u16),
    StructField(FieldIndex),
    EnumField {
        variant: VariantIndex,
        field: FieldIndex,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MetadataPath(Arc<[ProjectionStep]>);

impl MetadataPath {
    pub(crate) fn appended(&self, step: ProjectionStep) -> Self {
        let mut output = self.0.to_vec();
        output.push(step);
        Self(output.into())
    }

    pub(crate) fn steps(&self) -> &[ProjectionStep] {
        &self.0
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}
