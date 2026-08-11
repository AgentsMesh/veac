mod kind;
mod timeline;

use veac_plan::canonical::{BlendMode, ItemId};
use veac_plan::{ResolvedClip, ResolvedSequence, ResolvedTrack, ResolvedTransition};

use super::error::{diagnostic, CodegenErrorKind};
use super::{blend, layer, time, CodegenErrors, EmitContext};

pub(super) fn compose(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    base: String,
    track: &ResolvedTrack,
    transition: &ResolvedTransition,
    base_is_opaque: bool,
) -> Result<String, CodegenErrors> {
    let outgoing = endpoint_clip(track, transition, &transition.outgoing_clip_id, "outgoing")?;
    let incoming = endpoint_clip(track, transition, &transition.incoming_clip_id, "incoming")?;
    let outgoing = endpoint(context, sequence, outgoing)?;
    let incoming = endpoint(context, sequence, incoming)?;
    let outgoing = outgoing_side(context, &outgoing, transition);
    let incoming = incoming_side(context, &incoming, transition);
    let shifted = timeline::render(context, &outgoing, &incoming, transition);
    Ok(blend::composite_with_base(
        context,
        base,
        &shifted,
        blend::Placement {
            start: time::seconds(transition.record_window.start),
            end: time::end(transition.record_window),
            mode: BlendMode::Normal,
        },
        base_is_opaque,
    ))
}

fn endpoint(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
) -> Result<String, CodegenErrors> {
    let visual = clip
        .visual
        .as_ref()
        .ok_or_else(|| invalid_clip(clip, "transition endpoint has no visual properties"))?;
    layer::render(context, sequence, clip, visual)
}

fn outgoing_side(
    context: &mut EmitContext<'_>,
    input: &str,
    transition: &ResolvedTransition,
) -> String {
    let range = transition.outgoing_range;
    context.graph.filter(
        &[input],
        format!(
            "trim=start={}:duration={},setpts=PTS-STARTPTS",
            time::seconds(range.start),
            time::seconds(range.duration),
        ),
        "transitionoutv",
    )
}

fn incoming_side(
    context: &mut EmitContext<'_>,
    input: &str,
    transition: &ResolvedTransition,
) -> String {
    let range = transition.incoming_range;
    context.graph.filter(
        &[input],
        format!(
            "trim=start={}:duration={},setpts=PTS-STARTPTS",
            time::seconds(range.start),
            time::seconds(range.duration),
        ),
        "transitioninv",
    )
}

fn find_clip<'a>(track: &'a ResolvedTrack, id: &ItemId) -> Option<&'a ResolvedClip> {
    track.clips.iter().find(|clip| clip.id == *id)
}

fn endpoint_clip<'a>(
    track: &'a ResolvedTrack,
    transition: &ResolvedTransition,
    id: &ItemId,
    side: &str,
) -> Result<&'a ResolvedClip, CodegenErrors> {
    match find_clip(track, id) {
        Some(clip) => Ok(clip),
        None => Err(CodegenErrors::one(diagnostic(
            CodegenErrorKind::InvalidPlan,
            "TRANSITION_PLAN_INVALID",
            Some(id.to_string()),
            format!("{side} clip is missing for {:?}", transition.kind),
        ))),
    }
}

fn invalid_clip(clip: &ResolvedClip, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "TRANSITION_SOURCE_INVALID",
        Some(clip.id.to_string()),
        message,
    ))
}
