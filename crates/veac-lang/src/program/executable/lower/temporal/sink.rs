use crate::program::executable::ExecutableTemporalSink;
use veac_ir::{Project, TemporalBindingId};

use super::ExecutableLowerError;

mod apply;
mod clip;
mod context;
pub(super) use context::Context;

pub(super) fn context(
    project: &Project,
    target: &ExecutableTemporalSink,
) -> Result<Context, ExecutableLowerError> {
    context::resolve(project, target)
}

pub(super) fn attach(
    project: &mut Project,
    target: &ExecutableTemporalSink,
    binding_id: TemporalBindingId,
) -> Result<(), ExecutableLowerError> {
    match target {
        ExecutableTemporalSink::Clip { .. }
        | ExecutableTemporalSink::ClipMask { .. }
        | ExecutableTemporalSink::ClipText { .. }
        | ExecutableTemporalSink::ClipEffect { .. } => clip::attach(project, target, binding_id),
        ExecutableTemporalSink::ApplyOpacity { .. }
        | ExecutableTemporalSink::ApplyMask { .. }
        | ExecutableTemporalSink::ApplyEffect { .. } => apply::attach(project, target, binding_id),
    }
}
