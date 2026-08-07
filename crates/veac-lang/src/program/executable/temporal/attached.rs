use crate::program::executable::lower::ExecutableLowerError;
use crate::program::expression::runtime::domain_graph::{
    FrozenDomainGraph, FrozenTemporalAttachment,
};
use crate::program::expression::{ClosureValue, ResidualizationRequest, TemporalTargetKind, Value};
use crate::program::model::{TemporalApplyPath, TemporalItemPath, TemporalTarget};
use veac_ir::TemporalBindingId;

use super::ExecutableTemporalSink;

mod identity;

pub(in crate::program::executable) struct AttachedTemporalLeaf<'a> {
    sink: ExecutableTemporalSink,
    binding_id: TemporalBindingId,
    animation: &'a ClosureValue,
    request: ResidualizationRequest,
}

pub(in crate::program::executable) fn compile(
    graph: &FrozenDomainGraph,
) -> Result<Vec<AttachedTemporalLeaf<'_>>, ExecutableLowerError> {
    graph
        .temporal_attachments()
        .map(|attachment| {
            let (property, _) = attachment.kind().parts();
            let target = target(attachment)?;
            let sink = super::authored::sink::lower(&target, property)
                .map_err(|message| error("EXECUTABLE_TEMPORAL_TARGET", message))?;
            let content = attachment.animation().content_digest();
            Ok(AttachedTemporalLeaf {
                binding_id: identity::binding(&sink, content),
                request: identity::request(&sink, attachment, content)?,
                sink,
                animation: attachment.animation(),
            })
        })
        .collect()
}

impl AttachedTemporalLeaf<'_> {
    pub(in crate::program::executable) fn sink(&self) -> &ExecutableTemporalSink {
        &self.sink
    }

    pub(in crate::program::executable) fn binding_id(&self) -> &TemporalBindingId {
        &self.binding_id
    }

    pub(in crate::program::executable) fn animation(&self) -> &ClosureValue {
        self.animation
    }

    pub(in crate::program::executable) fn request(&self) -> &ResidualizationRequest {
        &self.request
    }
}

fn target(value: FrozenTemporalAttachment<'_>) -> Result<TemporalTarget, ExecutableLowerError> {
    let (property, kind) = value.kind().parts();
    let path = value.owner().logical_path().ok_or_else(|| {
        error(
            "EXECUTABLE_TEMPORAL_OWNER",
            "animation owner has no complete logical path",
        )
    })?;
    let selectors = value.selectors();
    let target = match kind {
        TemporalTargetKind::Clip => TemporalTarget::Clip(item_path(&path)?),
        TemporalTargetKind::Text => TemporalTarget::Text(item_path(&path)?),
        TemporalTargetKind::ClipMask => TemporalTarget::ClipMask {
            clip: item_path(&path)?,
            mask_index: ordinal(selectors)?,
        },
        TemporalTargetKind::ClipEffect => TemporalTarget::ClipEffect {
            clip: item_path(&path)?,
            effect: identifier(selectors, 0)?.to_owned(),
            parameter: identifier(selectors, 1)?.to_owned(),
        },
        TemporalTargetKind::Apply => TemporalTarget::Apply(apply_path(&path)?),
        TemporalTargetKind::ApplyMask => TemporalTarget::ApplyMask {
            apply: apply_path(&path)?,
            mask_index: ordinal(selectors)?,
        },
        TemporalTargetKind::ApplyEffect => TemporalTarget::ApplyEffect {
            apply: apply_path(&path)?,
            stage: identifier(selectors, 0)?.to_owned(),
            effect: identifier(selectors, 1)?.to_owned(),
            parameter: identifier(selectors, 2)?.to_owned(),
        },
    };
    debug_assert!(super::authored::sink::lower(&target, property).is_ok());
    Ok(target)
}

fn item_path(path: &[&str]) -> Result<TemporalItemPath, ExecutableLowerError> {
    let [project, sequence, layer, item] = path else {
        return Err(error(
            "EXECUTABLE_TEMPORAL_OWNER",
            "Item animation owner path must have four segments",
        ));
    };
    Ok(TemporalItemPath {
        project: (*project).into(),
        sequence: (*sequence).into(),
        layer: (*layer).into(),
        item: (*item).into(),
    })
}

fn apply_path(path: &[&str]) -> Result<TemporalApplyPath, ExecutableLowerError> {
    let [project, sequence, apply] = path else {
        return Err(error(
            "EXECUTABLE_TEMPORAL_OWNER",
            "Apply animation owner path must have three segments",
        ));
    };
    Ok(TemporalApplyPath {
        project: (*project).into(),
        sequence: (*sequence).into(),
        apply: (*apply).into(),
    })
}

fn ordinal(values: &[Value]) -> Result<u32, ExecutableLowerError> {
    match values.first() {
        Some(Value::Integer(value)) => u32::try_from(*value)
            .map_err(|_| error("EXECUTABLE_TEMPORAL_SELECTOR", "mask ordinal must fit u32")),
        _ => Err(error(
            "EXECUTABLE_TEMPORAL_SELECTOR",
            "mask ordinal is not an int",
        )),
    }
}

fn identifier(values: &[Value], index: usize) -> Result<&str, ExecutableLowerError> {
    match values.get(index) {
        Some(Value::Identifier(value)) => Ok(value),
        _ => Err(error(
            "EXECUTABLE_TEMPORAL_SELECTOR",
            "effect selector is not an identifier",
        )),
    }
}

fn error(reason: &'static str, message: impl Into<String>) -> ExecutableLowerError {
    ExecutableLowerError::lower(reason, message)
}

#[cfg(test)]
mod tests;
