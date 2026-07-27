use veac_plan::canonical::*;
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedClipSource, ResolvedRenderPlan};

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    report(check, usage(plan), plan.header.source.project_id.as_str());
}

fn usage(plan: &ResolvedRenderPlan) -> RenderStructureUsage {
    let mut total = RenderStructureUsage::default();
    for track in plan.sequences.iter().flat_map(|value| &value.tracks) {
        total.tracks = total.tracks.saturating_add(1);
        for clip in &track.clips {
            total.add(clip_usage(clip));
        }
    }
    total
}

fn clip_usage(clip: &ResolvedClip) -> RenderStructureUsage {
    let mut value = RenderStructureUsage {
        clips: 1,
        effects: count(clip.effects.len()),
        caption_cues: u64::from(matches!(clip.source, ResolvedClipSource::Caption { .. })),
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
        for parameter in effect.parameters.values() {
            if let ParameterValue::NumberCurve { value: curve } = parameter {
                value.keyframes = value.keyframes.saturating_add(keys(curve));
            }
        }
    }
    if let Some(mapping) = clip.source_mapping.as_ref() {
        if let veac_plan::ResolvedSourceTimeMap::Curve { segments } = &mapping.time_map {
            value.source_curve_segments = count(segments.len());
        }
    }
    value.keyframes = value.keyframes.saturating_add(text_keys(&clip.source));
    value
}

fn visual_keys(visual: &EffectiveVisualProperties) -> u64 {
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

fn text_keys(source: &ResolvedClipSource) -> u64 {
    let content = match source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            content
        }
        _ => return 0,
    };
    let Some(animation) = &content.style.animation else {
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

fn report(check: &mut Check, usage: RenderStructureUsage, project_id: &str) {
    let limits = [
        (
            usage.tracks,
            MAX_TOTAL_TRACKS,
            "PLAN_BUDGET_TRACKS",
            "render plan tracks exceed the execution budget",
        ),
        (
            usage.clips,
            MAX_TOTAL_CLIPS,
            "PLAN_BUDGET_CLIPS",
            "render plan clips exceed the execution budget",
        ),
        (
            usage.effects,
            MAX_TOTAL_EFFECTS,
            "PLAN_BUDGET_EFFECTS",
            "render plan effects exceed the execution budget",
        ),
        (
            usage.masks,
            MAX_TOTAL_MASKS,
            "PLAN_BUDGET_MASKS",
            "render plan masks exceed the execution budget",
        ),
        (
            usage.keyframes,
            MAX_TOTAL_KEYFRAMES,
            "PLAN_BUDGET_KEYFRAMES",
            "render plan keyframes exceed the execution budget",
        ),
        (
            usage.source_curve_segments,
            MAX_TOTAL_SOURCE_CURVE_SEGMENTS,
            "PLAN_BUDGET_SOURCE_CURVE_SEGMENTS",
            "render plan source-time segments exceed the execution budget",
        ),
        (
            usage.caption_cues,
            MAX_TOTAL_CAPTION_CUES,
            "PLAN_BUDGET_CAPTION_CUES",
            "render plan caption cues exceed the execution budget",
        ),
    ];
    for (actual, limit, code, message) in limits {
        if actual > limit {
            check.push(code, Some(project_id.to_owned()), message);
        }
    }
}

#[cfg(test)]
#[path = "structure/tests.rs"]
mod tests;
