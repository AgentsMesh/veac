use std::mem::{size_of, size_of_val};

use super::super::metadata::{
    CallableContract, DeferredCall, DeferredUse, MetadataPath, ProjectionContract,
};
use super::super::{CoreValueMetadata, FunctionSummary};

pub(super) fn summary_payload(summary: &FunctionSummary) -> Option<usize> {
    let mut bytes = payload(&summary.result)?;
    bytes = bytes.checked_add(
        summary
            .deferred_effects
            .len()
            .checked_mul(size_of::<std::sync::Arc<DeferredCall>>())?,
    )?;
    summary
        .deferred_effects
        .iter()
        .try_fold(bytes, |bytes, call| {
            bytes.checked_add(deferred_call_payload(call)?)
        })
}

pub(super) fn payload(metadata: &CoreValueMetadata) -> Option<usize> {
    let mut bytes = dependency_bytes(metadata)?;
    if let Some(callable) = &metadata.callable {
        bytes = bytes.checked_add(callable_payload(callable)?)?;
    }
    bytes = bytes.checked_add(projection_payload(&metadata.projection)?)?;
    bytes = bytes.checked_add(
        metadata
            .deferred
            .len()
            .checked_mul(size_of::<DeferredUse>())?,
    )?;
    metadata.deferred.iter().try_fold(bytes, |bytes, usage| {
        bytes
            .checked_add(path_payload(&usage.path)?)?
            .checked_add(deferred_call_payload(&usage.call)?)
    })
}

fn callable_payload(callable: &CallableContract) -> Option<usize> {
    match callable {
        CallableContract::Closure { summary, captures } => {
            let mut bytes = size_of::<FunctionSummary>().checked_add(summary_payload(summary)?)?;
            bytes =
                bytes.checked_add(captures.len().checked_mul(size_of::<CoreValueMetadata>())?)?;
            captures
                .iter()
                .try_fold(bytes, |bytes, value| bytes.checked_add(payload(value)?))
        }
        CallableContract::Join(values) => {
            let bytes = values.len().checked_mul(size_of::<CallableContract>())?;
            values.iter().try_fold(bytes, |bytes, value| {
                bytes.checked_add(callable_payload(value)?)
            })
        }
        CallableContract::Deferred { call, path } => {
            deferred_call_payload(call)?.checked_add(path_payload(path)?)
        }
        CallableContract::Binding { path, .. } => path_payload(path),
        CallableContract::Bound(_) => Some(0),
        CallableContract::Impossible => Some(0),
    }
}

fn projection_payload(value: &ProjectionContract) -> Option<usize> {
    match value {
        ProjectionContract::Opaque | ProjectionContract::Impossible => Some(0),
        ProjectionContract::Binding { path, .. } => path_payload(path),
        ProjectionContract::Deferred { call, path } => {
            deferred_call_payload(call)?.checked_add(path_payload(path)?)
        }
        ProjectionContract::Known(values) => {
            let bytes = values.len().checked_mul(size_of::<(
                super::super::metadata::ProjectionStep,
                CoreValueMetadata,
            )>())?;
            values.iter().try_fold(bytes, |bytes, (_, value)| {
                bytes.checked_add(payload(value)?)
            })
        }
        ProjectionContract::Alternatives(values) => {
            let bytes = values.len().checked_mul(size_of::<CoreValueMetadata>())?;
            values
                .iter()
                .try_fold(bytes, |bytes, value| bytes.checked_add(payload(value)?))
        }
    }
}

fn path_payload(path: &MetadataPath) -> Option<usize> {
    path.len()
        .checked_mul(size_of::<super::super::metadata::ProjectionStep>())
}

fn deferred_call_payload(call: &DeferredCall) -> Option<usize> {
    let mut bytes = size_of_val(call).checked_add(callable_payload(&call.callee)?)?;
    bytes = bytes.checked_add(
        call.arguments
            .len()
            .checked_mul(size_of::<CoreValueMetadata>())?,
    )?;
    for argument in call.arguments.iter() {
        bytes = bytes.checked_add(payload(argument)?)?;
    }
    bytes.checked_add(super::value_type_bytes(&call.result_type)?)
}

fn dependency_bytes(metadata: &CoreValueMetadata) -> Option<usize> {
    metadata
        .shape_dependencies()
        .retained_payload_bytes()?
        .checked_add(metadata.leaf_dependencies().retained_payload_bytes()?)
}
