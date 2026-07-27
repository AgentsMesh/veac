use veac_plan::canonical::{AudioCrossfade, AudioFadeCurve};
use veac_plan::ResolvedClip;

use super::{time, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    mut input: String,
    value: Option<AudioCrossfade>,
) -> String {
    let Some(value) = value else { return input };
    let curve = match value.curve {
        AudioFadeCurve::Linear => "tri",
        AudioFadeCurve::EqualPower => "qsin",
        AudioFadeCurve::Exponential => "exp",
    };
    if value.fade_in.value > 0 {
        input = context.graph.filter(
            &[&input],
            format!(
                "afade=t=in:st=0:d={}:curve={curve}",
                time::seconds(value.fade_in)
            ),
            "clipfadeina",
        );
    }
    if value.fade_out.value > 0 {
        let start = veac_plan::canonical::RationalTime {
            value: clip.record_range.duration.value - value.fade_out.value,
            timescale: clip.record_range.duration.timescale,
        };
        input = context.graph.filter(
            &[&input],
            format!(
                "afade=t=out:st={}:d={}:curve={curve}",
                time::seconds(start),
                time::seconds(value.fade_out)
            ),
            "clipfadeouta",
        );
    }
    input
}
