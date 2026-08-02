use veac_plan::canonical::MaterialKind;
use veac_plan::{
    PlanInputId, ResolvedClip, ResolvedClipSource, ResolvedInput, ResolvedInputKind,
    ResolvedMulticamAngle, ResolvedRenderPlan,
};

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        let valid = match &clip.source {
            ResolvedClipSource::Media { .. } | ResolvedClipSource::FreezeFrame { .. } => {
                trusted_media(plan, clip).is_some()
            }
            ResolvedClipSource::Multicam { source } => source
                .angles
                .iter()
                .all(|angle| trusted_angle(plan, angle, clip.audio.is_some()).is_some()),
            _ => true,
        };
        if !valid {
            check.push(
                "PLAN_INPUT_STREAM_INVALID",
                Some(clip.id.to_string()),
                "clip stream selections do not match typed facts on one unique media input",
            );
        }
    }
}

pub(super) fn trusted_media<'a>(
    plan: &'a ResolvedRenderPlan,
    clip: &ResolvedClip,
) -> Option<&'a ResolvedInput> {
    match &clip.source {
        ResolvedClipSource::Media {
            input_id,
            video_stream,
            audio_stream,
            ..
        } => {
            let input = unique_input(plan, input_id)?;
            let kind = media_kind(input)?;
            let video_used = clip.visual.is_some();
            let audio_used = clip.audio.is_some();
            if video_stream.is_some() != video_used
                || audio_stream.is_some() != audio_used
                || (video_used && !matches!(kind, MaterialKind::Video | MaterialKind::Image))
                || (audio_used && !matches!(kind, MaterialKind::Video | MaterialKind::Audio))
                || !selected_video(input, video_stream.as_ref())
                || !selected_audio(input, audio_stream.as_ref())
            {
                return None;
            }
            Some(input)
        }
        ResolvedClipSource::FreezeFrame {
            input_id,
            video_stream,
            ..
        } => {
            let input = unique_input(plan, input_id)?;
            let kind = media_kind(input)?;
            (matches!(kind, MaterialKind::Video | MaterialKind::Image)
                && selected_video(input, Some(video_stream)))
            .then_some(input)
        }
        _ => None,
    }
}

pub(super) fn trusted_angle<'a>(
    plan: &'a ResolvedRenderPlan,
    angle: &ResolvedMulticamAngle,
    audio_used: bool,
) -> Option<&'a ResolvedInput> {
    let input = unique_input(plan, &angle.input_id)?;
    let kind = media_kind(input)?;
    if kind != MaterialKind::Video
        || (audio_used && angle.audio_stream.is_none())
        || !selected_video(input, Some(&angle.video_stream))
        || !selected_audio(input, angle.audio_stream.as_ref())
    {
        return None;
    }
    Some(input)
}

fn unique_input<'a>(plan: &'a ResolvedRenderPlan, id: &PlanInputId) -> Option<&'a ResolvedInput> {
    let mut values = plan.inputs.iter().filter(|input| input.id == *id);
    let input = values.next()?;
    values.next().is_none().then_some(input)
}

fn media_kind(input: &ResolvedInput) -> Option<MaterialKind> {
    input.probe.as_ref()?;
    match input.kind {
        ResolvedInputKind::Media { material_kind } => Some(material_kind),
        _ => None,
    }
}

fn selected_video(
    input: &ResolvedInput,
    selected: Option<&veac_plan::canonical::StreamSelection>,
) -> bool {
    selected.is_none_or(|value| {
        input
            .video
            .as_ref()
            .is_some_and(|facts| facts.selection == *value)
    })
}

fn selected_audio(
    input: &ResolvedInput,
    selected: Option<&veac_plan::canonical::StreamSelection>,
) -> bool {
    selected.is_none_or(|value| {
        input
            .audio
            .as_ref()
            .is_some_and(|facts| facts.selection == *value)
    })
}
