use std::sync::Arc;

use super::{BindingRoot, CoreValueMetadata, FunctionSummary, MetadataPath};
use crate::program::expression::{FunctionEffect, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CallableContract {
    Closure {
        summary: Arc<FunctionSummary>,
        captures: Arc<[CoreValueMetadata]>,
    },
    Binding {
        root: BindingRoot,
        path: MetadataPath,
        fallback: FunctionEffect,
    },
    Bound(FunctionEffect),
    Join(Arc<[CallableContract]>),
    Deferred {
        call: Arc<DeferredCall>,
        path: MetadataPath,
    },
    Impossible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeferredCall {
    pub(crate) callee: CallableContract,
    pub(crate) arguments: Arc<[CoreValueMetadata]>,
    pub(crate) result_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeferredUse {
    pub(crate) call: Arc<DeferredCall>,
    pub(crate) path: MetadataPath,
    pub(crate) shape: bool,
    pub(crate) leaf: bool,
}

impl DeferredUse {
    pub(crate) fn new(call: Arc<DeferredCall>, path: MetadataPath) -> Self {
        Self {
            call,
            path,
            shape: false,
            leaf: false,
        }
    }

    pub(crate) fn with_axes(&self, shape: bool, leaf: bool) -> Self {
        Self {
            call: Arc::clone(&self.call),
            path: self.path.clone(),
            shape: shape && (self.shape || self.leaf),
            leaf: leaf && (self.shape || self.leaf),
        }
    }
}

impl CallableContract {
    pub(super) fn bind(
        &self,
        parameters: &[CoreValueMetadata],
        captures: &[CoreValueMetadata],
    ) -> Option<Self> {
        match self {
            Self::Binding {
                root,
                path,
                fallback,
            } => binding(*root, parameters, captures)
                .and_then(|value| value.project_path(path, true))
                .and_then(|value| value.callable)
                .or(Some(Self::Bound(*fallback))),
            Self::Closure {
                summary,
                captures: bound,
            } => Some(Self::Closure {
                summary: Arc::clone(summary),
                captures: bound
                    .iter()
                    .map(|value| value.bind(parameters, captures))
                    .collect::<Option<Vec<_>>>()?
                    .into(),
            }),
            Self::Join(values) => join(
                values
                    .iter()
                    .map(|value| value.bind(parameters, captures))
                    .collect::<Option<Vec<_>>>()?,
            ),
            Self::Deferred { call, path } => {
                call.bind(parameters, captures)?
                    .project_path(path, true)?
                    .callable
            }
            Self::Bound(effect) => Some(Self::Bound(*effect)),
            Self::Impossible => Some(Self::Impossible),
        }
    }
}

impl DeferredCall {
    pub(super) fn bind(
        &self,
        parameters: &[CoreValueMetadata],
        captures: &[CoreValueMetadata],
    ) -> Option<CoreValueMetadata> {
        let callee = self.callee.bind(parameters, captures)?;
        let arguments = self
            .arguments
            .iter()
            .map(|value| value.bind(parameters, captures))
            .collect::<Option<Vec<_>>>()?;
        CoreValueMetadata::invoke_contract(callee, &arguments, &self.result_type)
    }
}

pub(super) fn join(values: impl IntoIterator<Item = CallableContract>) -> Option<CallableContract> {
    let mut output = Vec::new();
    let mut impossible = false;
    for value in values {
        let candidates: &[CallableContract] = match &value {
            CallableContract::Join(values) => values,
            _ => std::slice::from_ref(&value),
        };
        for candidate in candidates {
            if matches!(candidate, CallableContract::Impossible) {
                impossible = true;
                continue;
            }
            if !output.contains(candidate) {
                output.push(candidate.clone());
            }
        }
    }
    match output.len() {
        0 if impossible => Some(CallableContract::Impossible),
        0 => None,
        1 => output.pop(),
        _ => Some(CallableContract::Join(output.into())),
    }
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
