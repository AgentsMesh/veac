use veac_plan::canonical::AlphaMode;
use veac_plan::ResolvedSequence;

use super::{canvas::Canvas, time, visual, CodegenErrors, EmitContext};

pub(super) fn build_entry(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
) -> Result<String, CodegenErrors> {
    let background = if context.alpha == AlphaMode::Straight {
        "black@0"
    } else {
        "black"
    };
    build_video(context, sequence, background)
}

pub(super) fn build_nested(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
) -> Result<String, CodegenErrors> {
    build_video(context, sequence, "black@0")
}

fn build_video(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    background: &str,
) -> Result<String, CodegenErrors> {
    let previous = context.canvas;
    context.canvas = Canvas::from_sequence(sequence);
    let result = build_sequence(context, sequence, background);
    context.canvas = previous;
    result
}

fn build_sequence(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    background: &str,
) -> Result<String, CodegenErrors> {
    let canvas = context.canvas;
    let base = context.graph.source(
        format!(
            "color=c={background}:s={}x{}:r={}/{}:d={},format=gbrap16le",
            canvas.width,
            canvas.height,
            canvas.frame_rate.numerator,
            canvas.frame_rate.denominator,
            time::seconds(sequence.duration)
        ),
        "base",
    );
    visual::compose(context, sequence, base)
}

pub(super) fn conform_output(
    context: &mut EmitContext<'_>,
    input: String,
    resolved: &ResolvedSequence,
) -> String {
    let sequence = Canvas::from_sequence(resolved);
    let output = &context.plan.output;
    let mut filters = Vec::new();
    if sequence.width != output.width || sequence.height != output.height {
        filters.push(format!(
            concat!(
                "scale={width}:{height}:force_original_aspect_ratio=decrease,",
                "pad={width}:{height}:(ow-iw)/2:(oh-ih)/2,setsar=1"
            ),
            width = output.width,
            height = output.height
        ));
    }
    if sequence.frame_rate != output.frame_rate {
        filters.push(format!(
            "fps={}/{}",
            output.frame_rate.numerator, output.frame_rate.denominator
        ));
    }
    if filters.is_empty() {
        input
    } else {
        context
            .graph
            .filter(&[&input], filters.join(","), "outputconformv")
    }
}
