use veac_plan::canonical::VideoStreamInfo;
use veac_plan::ResolvedClip;

use crate::emitter::{time, video_source, CodegenErrors, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    info: Option<&VideoStreamInfo>,
) -> Result<String, CodegenErrors> {
    let mut label = context.graph.filter(
        &[raw],
        format!(
            "loop=loop=-1:size=1:start=0,trim=start=0:duration={},setpts=PTS-STARTPTS",
            time::seconds(clip.record_range.duration),
        ),
        "staticv",
    );
    if let Some(info) = info {
        label = video_source::normalize_geometry(context, &label, info);
    }
    Ok(label)
}
