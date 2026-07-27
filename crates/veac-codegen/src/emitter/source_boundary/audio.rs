use veac_artifact::SourceClock;
use veac_plan::canonical::AudioOutput;
use veac_plan::ResolvedClip;

use super::{invalid, Padding};
use crate::emitter::{time, CodegenErrors, EmitContext};

pub(super) fn pad(
    context: &mut EmitContext<'_>,
    raw: &str,
    clip: &ResolvedClip,
    clock: SourceClock,
    padding: Padding,
    output: &AudioOutput,
) -> Result<(String, SourceClock), CodegenErrors> {
    let logical = clock
        .logical_range()
        .ok_or_else(|| invalid(clip, "audio boundary clock is unbounded"))?;
    let before = time::samples(padding.before, output.sample_rate)
        .ok_or_else(|| invalid(clip, "source head padding is not sample aligned"))?;
    let after = time::samples(padding.after, output.sample_rate)
        .ok_or_else(|| invalid(clip, "source tail padding is not sample aligned"))?;
    let mut filters = vec![format!(
        "atrim=start={}:duration={},asetpts=PTS-STARTPTS",
        time::seconds(clock.physical_start()),
        time::seconds(logical.duration),
    )];
    if before > 0 {
        filters.push(format!("adelay={before}S:all=1"));
    }
    if after > 0 {
        filters.push(format!("apad=pad_len={after}"));
    }
    filters.push("asetpts=PTS-STARTPTS".to_owned());
    let label = context.graph.filter(&[raw], filters.join(","), "boundpada");
    let clock = clock
        .extended(padding.before, padding.after)
        .map_err(|_| invalid(clip, "audio boundary clock extension failed"))?;
    Ok((label, clock))
}
