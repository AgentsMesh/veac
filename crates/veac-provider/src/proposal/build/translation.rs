use std::collections::BTreeSet;

use veac_ir::{ClipSource, EditOperation, Precondition, ProjectEnvelope};

use super::support::{self, BuiltApplication};
use crate::{
    ProposalEvidence, ProviderResult, TranslatedUnit, TranslationApplication, TranslationResult,
};

pub(super) fn build(
    project: &ProjectEnvelope,
    result: &TranslationResult,
    context: &TranslationApplication,
) -> ProviderResult<BuiltApplication> {
    validate_bindings(result, context)?;
    let mut operations = Vec::with_capacity(result.units.len());
    let mut evidence = Vec::with_capacity(result.units.len());
    let mut track_ids = BTreeSet::new();
    for (unit, binding) in result.units.iter().zip(&context.bindings) {
        let (track, clip) = support::clip(project, &binding.clip_id)?;
        if track.state.locked
            || !matches!(
                clip.source,
                ClipSource::Text { .. } | ClipSource::Caption { .. }
            )
        {
            return support::invalid("translation target must be an unlocked text or caption clip");
        }
        validate_range(unit, clip, project.project.timebase)?;
        let operation_index = support::index(operations.len())?;
        let operation = EditOperation::SetText {
            clip_id: binding.clip_id.clone(),
            text: unit.text.clone(),
        };
        evidence.push(ProposalEvidence::TranslationUnit {
            operation: crate::OperationBinding::new(operation_index, &operation)?,
            unit_id: unit.id.clone(),
        });
        operations.push(operation);
        track_ids.insert(track.id.clone());
    }
    let mut built = BuiltApplication::new(operations, evidence);
    built.preconditions = track_ids
        .into_iter()
        .map(|track_id| Precondition::TrackUnlocked { track_id })
        .collect();
    Ok(built)
}

fn validate_bindings(
    result: &TranslationResult,
    context: &TranslationApplication,
) -> ProviderResult<()> {
    if result.units.is_empty() || result.units.len() != context.bindings.len() {
        return support::invalid("translation output and application bindings must be non-empty");
    }
    let mut clips = BTreeSet::new();
    for (index, (unit, binding)) in result.units.iter().zip(&context.bindings).enumerate() {
        if unit.id != binding.unit_id
            || (index > 0 && context.bindings[index - 1].unit_id >= binding.unit_id)
            || !clips.insert(binding.clip_id.clone())
        {
            return support::invalid(
                "translation bindings must exactly match sorted units and unique clips",
            );
        }
    }
    Ok(())
}

fn validate_range(
    unit: &TranslatedUnit,
    clip: &veac_ir::Clip,
    timebase: u32,
) -> ProviderResult<()> {
    let Some(range) = unit.range else {
        return Ok(());
    };
    if super::super::time::range_to_timebase(range, timebase)? != clip.record_range {
        return support::invalid("translated unit range does not match its target clip");
    }
    Ok(())
}
