use veac_artifact::SourceClock;
use veac_plan::ResolvedClip;

use super::{invalid, Padding};
use crate::emitter::{time, CodegenErrors, EmitContext};

pub(super) fn pad(
    context: &mut EmitContext<'_>,
    raw: &str,
    clip: &ResolvedClip,
    clock: SourceClock,
    padding: Padding,
) -> Result<(String, SourceClock), CodegenErrors> {
    let logical = clock
        .logical_range()
        .ok_or_else(|| invalid(clip, "video boundary clock is unbounded"))?;
    let mut filters = vec![format!(
        "trim=start={}:duration={},setpts=PTS-STARTPTS",
        time::seconds(clock.physical_start()),
        time::seconds(logical.duration),
    )];
    let mut tpad = Vec::new();
    if padding.before.value > 0 {
        tpad.push(format!(
            "start_duration={}:start_mode=clone",
            time::seconds(padding.before)
        ));
    }
    if padding.after.value > 0 {
        tpad.push("stop=-1:stop_mode=clone".to_owned());
    }
    if !tpad.is_empty() {
        filters.push(format!("tpad={}", tpad.join(":")));
    }
    filters.push("setpts=PTS-STARTPTS".to_owned());
    let label = context.graph.filter(&[raw], filters.join(","), "boundpadv");
    let clock = clock
        .extended(padding.before, padding.after)
        .map_err(|_| invalid(clip, "video boundary clock extension failed"))?;
    Ok((label, clock))
}
