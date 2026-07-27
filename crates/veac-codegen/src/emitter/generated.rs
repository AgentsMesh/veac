use veac_plan::canonical::Generator;
use veac_plan::ResolvedClip;

use super::{time, EmitContext};

pub(super) fn video(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    generator: &Generator,
) -> String {
    match generator {
        Generator::Gradient { gradient } => {
            super::generated_gradient::render(context, clip, gradient)
        }
        Generator::Shape { shape } => super::generated_shape::render(context, clip, shape),
        _ => unreachable!("simple generators are emitted by source"),
    }
}

pub(super) fn high_precision_canvas(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    prefix: &str,
) -> String {
    let canvas = context.canvas;
    context.graph.source(
        format!(
            "color=c=black@0:s={}x{}:r={}/{}:d={},format=gbrap16le",
            canvas.width,
            canvas.height,
            canvas.frame_rate.numerator,
            canvas.frame_rate.denominator,
            time::seconds(clip.record_range.duration)
        ),
        prefix,
    )
}
