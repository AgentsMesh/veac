use crate::*;

use super::super::Validator;

pub(super) fn validate(
    validator: &mut Validator,
    project: &Project,
    temporal: &TemporalProgramLibrary,
) {
    report(validator, usage(project, temporal), project.id.as_str());
}

fn usage(project: &Project, temporal: &TemporalProgramLibrary) -> RenderStructureUsage {
    let mut total = RenderStructureUsage {
        temporal_programs: count(temporal.programs.len()),
        temporal_bindings: count(temporal.bindings.len()),
        temporal_nodes: temporal
            .programs
            .iter()
            .map(|program| count(program.nodes.len()))
            .fold(0, u64::saturating_add),
        temporal_provenance: count(temporal.provenance.len()),
        ..RenderStructureUsage::default()
    };
    for track in project.sequences.iter().flat_map(|value| &value.tracks) {
        total.tracks = total.tracks.saturating_add(1);
        for clip in &track.clips {
            total.add(clip_usage(clip));
        }
    }
    total
}

fn clip_usage(clip: &Clip) -> RenderStructureUsage {
    let mut value = RenderStructureUsage {
        clips: 1,
        effects: count(clip.effects.len()),
        caption_cues: u64::from(matches!(clip.source, ClipSource::Caption { .. })),
        ..RenderStructureUsage::default()
    };
    if let Some(visual) = &clip.visual {
        value.masks = count(visual.masks.len());
        value.keyframes = visual_keys(visual);
    }
    if let Some(audio) = &clip.audio {
        value.keyframes = value
            .keyframes
            .saturating_add(keys(&audio.gain))
            .saturating_add(keys(&audio.pan));
    }
    for effect in &clip.effects {
        for parameter in EffectParameter::ALL {
            if let Some(curve) = effect.effect.curve(parameter) {
                value.keyframes = value.keyframes.saturating_add(keys(curve));
            }
        }
    }
    if let Some(mapping) = &clip.source_mapping {
        if let SourceTimeMap::Curve { segments } = &mapping.time_map {
            value.source_curve_segments = count(segments.len());
        }
    }
    value.keyframes = value.keyframes.saturating_add(text_keys(&clip.source));
    value
}

fn visual_keys(visual: &VisualProperties) -> u64 {
    let transform = &visual.transform;
    let mut total = keys(&transform.position)
        .saturating_add(keys(&transform.scale))
        .saturating_add(keys(&transform.rotation_degrees))
        .saturating_add(transform.crop.as_ref().map_or(0, keys))
        .saturating_add(keys(&visual.opacity));
    for mask in &visual.masks {
        total = total
            .saturating_add(keys(&mask.position))
            .saturating_add(keys(&mask.scale))
            .saturating_add(keys(&mask.rotation_degrees))
            .saturating_add(keys(&mask.feather_pixels))
            .saturating_add(keys(&mask.expansion_pixels));
    }
    total
}

fn text_keys(source: &ClipSource) -> u64 {
    let style = match source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => style,
        _ => return 0,
    };
    let Some(animation) = &style.animation else {
        return 0;
    };
    keys(&animation.reveal)
        .saturating_add(
            animation
                .highlight
                .as_ref()
                .map_or(0, |highlight| keys(&highlight.progress)),
        )
        .saturating_add(keys(&animation.opacity))
        .saturating_add(keys(&animation.transform.position_offset))
        .saturating_add(keys(&animation.transform.scale))
        .saturating_add(keys(&animation.transform.rotation_degrees))
}

fn keys<T>(value: &Animatable<T>) -> u64 {
    value.keyframes().map_or(0, |items| count(items.len()))
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn report(validator: &mut Validator, usage: RenderStructureUsage, project_id: &str) {
    let limits = [
        (usage.tracks, MAX_TOTAL_TRACKS, "BUDGET_TRACKS", "tracks"),
        (usage.clips, MAX_TOTAL_CLIPS, "BUDGET_CLIPS", "clips"),
        (
            usage.effects,
            MAX_TOTAL_EFFECTS,
            "BUDGET_EFFECTS",
            "effects",
        ),
        (usage.masks, MAX_TOTAL_MASKS, "BUDGET_MASKS", "masks"),
        (
            usage.keyframes,
            MAX_TOTAL_KEYFRAMES,
            "BUDGET_KEYFRAMES",
            "keyframes",
        ),
        (
            usage.source_curve_segments,
            MAX_TOTAL_SOURCE_CURVE_SEGMENTS,
            "BUDGET_SOURCE_CURVE_SEGMENTS",
            "source-time curve segments",
        ),
        (
            usage.caption_cues,
            MAX_TOTAL_CAPTION_CUES,
            "BUDGET_CAPTION_CUES",
            "caption cues",
        ),
        (
            usage.temporal_programs,
            MAX_TOTAL_TEMPORAL_PROGRAMS,
            "BUDGET_TEMPORAL_PROGRAMS",
            "temporal programs",
        ),
        (
            usage.temporal_bindings,
            MAX_TOTAL_TEMPORAL_BINDINGS,
            "BUDGET_TEMPORAL_BINDINGS",
            "temporal bindings",
        ),
        (
            usage.temporal_nodes,
            MAX_TOTAL_TEMPORAL_NODES,
            "BUDGET_TEMPORAL_NODES",
            "temporal nodes",
        ),
        (
            usage.temporal_provenance,
            MAX_TOTAL_TEMPORAL_PROVENANCE,
            "BUDGET_TEMPORAL_PROVENANCE",
            "temporal provenance records",
        ),
    ];
    for (actual, limit, code, label) in limits {
        if actual > limit {
            validator.push(
                code,
                Some(project_id.to_owned()),
                "/project",
                format!("project {label} exceed the untrusted render execution budget"),
                Some("split the edit into smaller render projects"),
            );
        }
    }
}

#[cfg(test)]
#[path = "structure/tests.rs"]
mod tests;
