use crate::program::executable::ExecutableTemporalSink;
use veac_ir::{Clip, ClipSource, ItemId, MaterialId, Project, SequenceId};

use super::super::{error, ExecutableLowerError};

#[derive(Debug, Clone)]
pub(in crate::program::executable::lower::temporal) struct Context {
    pub(in crate::program::executable::lower::temporal) sequence_id: SequenceId,
    pub(in crate::program::executable::lower::temporal) item_id: Option<ItemId>,
    pub(in crate::program::executable::lower::temporal) source_material: Option<MaterialId>,
}

pub(super) fn resolve(
    project: &Project,
    target: &ExecutableTemporalSink,
) -> Result<Context, ExecutableLowerError> {
    if let Some(item_id) = target.item_id() {
        return clip(project, item_id);
    }
    let sequence_id = target.sequence_id().expect("apply sink has sequence owner");
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| &sequence.id == sequence_id)
        .ok_or_else(missing)?;
    let apply_id = match target {
        ExecutableTemporalSink::ApplyOpacity { apply_id, .. }
        | ExecutableTemporalSink::ApplyMask { apply_id, .. }
        | ExecutableTemporalSink::ApplyEffect { apply_id, .. } => apply_id,
        _ => unreachable!("item sinks returned above"),
    };
    sequence
        .applies
        .iter()
        .any(|apply| &apply.id == apply_id)
        .then(|| Context {
            sequence_id: sequence.id.clone(),
            item_id: None,
            source_material: None,
        })
        .ok_or_else(missing)
}

fn clip(project: &Project, item_id: &ItemId) -> Result<Context, ExecutableLowerError> {
    project
        .sequences
        .iter()
        .find_map(|sequence| {
            sequence
                .tracks
                .iter()
                .flat_map(|track| &track.clips)
                .find(|clip| &clip.id == item_id)
                .map(|clip| Context {
                    sequence_id: sequence.id.clone(),
                    item_id: Some(clip.id.clone()),
                    source_material: source_material(clip).cloned(),
                })
        })
        .ok_or_else(missing)
}

fn source_material(clip: &Clip) -> Option<&MaterialId> {
    match &clip.source {
        ClipSource::Media { material_id } | ClipSource::FreezeFrame { material_id, .. } => {
            Some(material_id)
        }
        _ => None,
    }
}

fn missing() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
        "the temporal host sink does not name an emitted canonical owner",
    )
}
