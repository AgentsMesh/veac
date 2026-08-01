use veac_plan::canonical::TrackMatteMode;
use veac_plan::{ResolvedClip, ResolvedMatte, ResolvedSequence};

use super::error::{diagnostic, CodegenErrorKind};
use super::{layer, time, CodegenErrors, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    target: String,
    matte: &ResolvedMatte,
) -> Result<String, CodegenErrors> {
    apply_window(
        context,
        sequence,
        &clip.id.to_string(),
        target,
        matte,
        clip.record_range,
    )
}

pub(super) fn apply_window(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    owner_id: &str,
    target: String,
    matte: &ResolvedMatte,
    window: veac_plan::canonical::TimeRange,
) -> Result<String, CodegenErrors> {
    let source_clip = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .find(|candidate| candidate.id == matte.source_clip_id)
        .ok_or_else(|| invalid(owner_id, "track matte source is absent"))?;
    let source_visual = source_clip
        .visual
        .as_ref()
        .ok_or_else(|| invalid(owner_id, "track matte source has no visuals"))?;
    let source_layer = layer::render(context, sequence, source_clip, source_visual)?;
    let offset = time::seconds_delta(source_clip.record_range.start, window.start);
    let source_layer = context.graph.filter(
        &[&source_layer],
        format!(
            "trim=start={offset}:duration={},setpts=PTS-STARTPTS",
            time::seconds(window.duration)
        ),
        "mattetrimv",
    );
    Ok(merge(
        context,
        &target,
        &source_layer,
        matte.mode,
        matte.invert,
    ))
}

fn merge(
    context: &mut EmitContext<'_>,
    target: &str,
    matte: &str,
    mode: TrackMatteMode,
    invert: bool,
) -> String {
    let (color, alpha) = context.graph.split(target, "mattetargetsplitv");
    let color = super::rgb_planes::without_alpha(&mut context.graph, &color, "mattetargetv");
    let alpha = context.graph.filter(
        &[&alpha],
        "format=rgba64le,alphaextract,format=gray16le",
        "mattetargetalphav",
    );
    let filter = match mode {
        TrackMatteMode::Alpha => "format=rgba64le,alphaextract,format=gray16le",
        TrackMatteMode::Luma => "format=gray16le",
    };
    let mut matte = context.graph.filter(&[matte], filter, "mattevaluev");
    if invert {
        matte = context.graph.filter(&[&matte], "negate", "matteinvertv");
    }
    let alpha = context
        .graph
        .filter(&[&alpha, &matte], "blend=all_mode=multiply", "mattealphav");
    super::alpha_merge::apply(context, &color, &alpha, "gbrap16le", "mattemergev")
}

fn invalid(owner_id: &str, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "MATTE_PLAN_INVALID",
        Some(owner_id.to_owned()),
        message,
    ))
}
