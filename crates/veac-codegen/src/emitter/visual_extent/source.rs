use veac_plan::{ResolvedClip, ResolvedClipSource, ResolvedRenderPlan};

pub(super) fn source_extent(plan: &ResolvedRenderPlan, clip: &ResolvedClip) -> Option<(u32, u32)> {
    match &clip.source {
        ResolvedClipSource::Media { input_id, .. }
        | ResolvedClipSource::FreezeFrame { input_id, .. } => input_extent(plan, input_id),
        ResolvedClipSource::Sequence { sequence_id } => plan
            .sequences
            .iter()
            .find(|value| value.id == *sequence_id)
            .map(|value| (value.settings.width, value.settings.height)),
        ResolvedClipSource::Multicam { source } => source
            .angles
            .iter()
            .filter_map(|angle| input_extent(plan, &angle.input_id))
            .reduce(|left, right| (left.0.max(right.0), left.1.max(right.1))),
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            text_extent(content)
        }
        ResolvedClipSource::Generated { .. } => None,
    }
}

fn input_extent(plan: &ResolvedRenderPlan, id: &veac_plan::PlanInputId) -> Option<(u32, u32)> {
    let info = plan
        .inputs
        .iter()
        .find(|value| value.id == *id)?
        .video
        .as_ref()
        .map(|video| &video.info)?;
    veac_plan::canonical::normalized_video_dimensions(info)
}

fn text_extent(content: &veac_plan::ResolvedText) -> Option<(u32, u32)> {
    let layout = content.styled()?.layout;
    Some((
        layout.box_width_pixels?.round() as u32,
        layout.box_height_pixels?.round() as u32,
    ))
}
