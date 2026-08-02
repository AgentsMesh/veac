use std::collections::BTreeSet;

use veac_artifact::ArtifactKind;
use veac_ir::{MaterialKind, ProjectEnvelope};

use super::media::{self, RequiredStream};
use super::support::{self, BuiltApplication};
use crate::{ProviderResult, SeparationApplication, VocalSeparationRequest, VocalSeparationResult};

pub(super) fn build(
    project: &ProjectEnvelope,
    request: &VocalSeparationRequest,
    result: &VocalSeparationResult,
    context: &SeparationApplication,
) -> ProviderResult<BuiltApplication> {
    if result.stems.len() != context.bindings.len() || result.stems.is_empty() {
        return support::invalid("separation bindings must cover every output stem");
    }
    let (source_track, source_clip) = media::source_clip(
        project,
        &context.source_clip_id,
        &request.audio,
        RequiredStream::Audio,
        true,
    )?;
    let expected_range = request
        .audio
        .range
        .ok_or_else(|| invalid("separation input requires an exact range"))?;
    let expected_range = media::normalized_range(project, expected_range)?;
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    let mut material_ids = BTreeSet::new();
    let mut clip_ids = BTreeSet::new();
    let mut artifact_keys = BTreeSet::new();
    for (stem, binding) in result.stems.iter().zip(&context.bindings) {
        if stem.kind != binding.kind
            || stem.label != binding.label
            || stem.audio.kind() != ArtifactKind::AudioStem
            || !material_ids.insert(binding.output.material.material.id.clone())
            || !clip_ids.insert(binding.output.clip_id.clone())
            || !artifact_keys.insert(stem.audio.record.key.clone())
        {
            return support::invalid("separation stem binding is not unique or exact");
        }
        let record_range = media::normalized_range(project, binding.output.record_range)?;
        let source_start = media::normalized_time(project, binding.output.source_start)?;
        if record_range != expected_range
            || binding.output.before_id.is_some() && binding.output.after_id.is_some()
        {
            return support::invalid("separation stem range or insertion anchor is invalid");
        }
        let track = media::audio_target(
            project,
            &binding.output.sequence_id,
            &binding.output.track_id,
        )?;
        media::output_material(
            project,
            &stem.audio,
            &binding.output.material,
            MaterialKind::Audio,
            RequiredStream::Audio,
            source_start,
            record_range.duration,
        )?;
        media::append_material(&mut built, &binding.output.material, &stem.audio)?;
        media::append_stem_clip(
            &mut built,
            &binding.output,
            &stem.audio,
            stem.kind,
            &stem.label,
            record_range,
            source_start,
        )?;
        media::add_track_precondition(&mut built, track);
    }
    built
        .preconditions
        .extend(media::source_preconditions(source_track, source_clip));
    Ok(built)
}

fn invalid(message: &str) -> crate::ProviderError {
    crate::ProviderError::new(crate::ProviderErrorKind::InvalidContract, message)
}
