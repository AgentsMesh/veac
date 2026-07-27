mod kind;

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
) -> Result<String, CodegenErrors> {
    let outgoing = endpoint_clip(track, transition, &transition.outgoing_clip_id, "outgoing")?;
    let incoming = endpoint_clip(track, transition, &transition.incoming_clip_id, "incoming")?;
    let outgoing = endpoint(context, sequence, outgoing)?;
    let incoming = endpoint(context, sequence, incoming)?;
    let outgoing = outgoing_side(context, &outgoing, transition);
    let incoming = incoming_side(context, &incoming, transition);
    let mixed = context.graph.filter(
        &[&outgoing, &incoming],
        format!(
            "{},format=rgba",
            kind::filter(
                &transition.kind,
                &time::seconds(transition.record_window.duration)
            )
        ),
        "transitionv",
    );
    let shifted = context.graph.filter(
        &[&mixed],
        format!(
            "setpts=PTS+{}/TB",
            time::seconds(transition.record_window.start)
        ),
        "transitionoffsetv",
    );
    Ok(blend::composite(
        context,
        base,
        &shifted,
        blend::Placement {
            x: "0",
            y: "0",
            start: time::seconds(transition.record_window.start),
            end: time::end(transition.record_window),
            mode: BlendMode::Normal,
            shadow: None,
        },
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
    let handle = transition.outgoing_handle;
    if handle.duration.value == 0 {
        let start = time::frame_window_start(handle.offset, context.canvas.frame_rate);
        return context.graph.filter(
            &[input],
            format!(
                "trim=start={start},reverse,trim=end_frame=1,setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop_duration={},trim=duration={}",
                time::seconds(transition.record_window.duration),
                time::seconds(transition.record_window.duration)
            ),
            "transitionoutv",
        );
    }
    context.graph.filter(
        &[input],
        format!(
            "trim=start={}:duration={},setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop_duration={},trim=duration={}",
            time::seconds(handle.offset),
            time::seconds(handle.duration),
            time::seconds(transition.incoming_handle.duration),
            time::seconds(transition.record_window.duration)
        ),
        "transitionoutv",
    )
}

fn incoming_side(
    context: &mut EmitContext<'_>,
    input: &str,
    transition: &ResolvedTransition,
) -> String {
    let handle = transition.incoming_handle;
    if handle.duration.value == 0 {
        return context.graph.filter(
            &[input],
            format!(
                "trim=end_frame=1,setpts=PTS-STARTPTS,tpad=start_mode=clone:start_duration={},trim=duration={}",
                time::seconds(transition.record_window.duration),
                time::seconds(transition.record_window.duration)
            ),
            "transitioninv",
        );
    }
    context.graph.filter(
        &[input],
        format!(
            "trim=start=0:duration={},setpts=PTS-STARTPTS,tpad=start_mode=clone:start_duration={},trim=duration={}",
            time::seconds(handle.duration),
            time::seconds(transition.outgoing_handle.duration),
            time::seconds(transition.record_window.duration)
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
