use veac_plan::canonical::RationalTime;
use veac_plan::ResolvedClip;

use super::{time, EmitContext};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct TransitionFades {
    pub fade_in: Option<RationalTime>,
    pub fade_out: Option<RationalTime>,
}

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    mut input: String,
    fades: TransitionFades,
) -> String {
    if let Some(duration) = fades.fade_in.filter(|duration| duration.value > 0) {
        input = context.graph.filter(
            &[&input],
            format!("afade=t=in:st=0:d={}", time::seconds(duration)),
            "transitionina",
        );
    }
    if let Some(duration) = fades.fade_out.filter(|duration| duration.value > 0) {
        let start = clip.record_range.duration.value - duration.value;
        input = context.graph.filter(
            &[&input],
            format!(
                "afade=t=out:st={}:d={}",
                time::seconds(RationalTime {
                    value: start,
                    timescale: duration.timescale,
                }),
                time::seconds(duration)
            ),
            "transitionouta",
        );
    }
    input
}
