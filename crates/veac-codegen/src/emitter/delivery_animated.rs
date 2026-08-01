use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{
    AlphaMode, AnimatedImage, Deliverable, GifAnimation, GifDither, GifPlayback,
};
use veac_plan::ResolvedRenderPlan;

use super::{
    output, sequence, time, BackendAction, BackendCommand, BackendOutput, BackendPhase,
    BackendProduct, BackendTask, CodegenErrors, EmitContext,
};

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &AnimatedImage,
) -> Result<BackendTask, CodegenErrors> {
    let AnimatedImage::Gif(settings) = settings;
    let mut context = EmitContext::new_visual(plan, bindings, deliverable, AlphaMode::Straight)?;
    let resolved = context
        .plan
        .sequences
        .iter()
        .find(|value| value.id == context.plan.entry_sequence_id)
        .ok_or_else(|| CodegenErrors::one(super::error::missing_entry(context.plan)))?;
    let video = sequence::build_entry(&mut context, resolved)?;
    let video = sequence::conform_output(&mut context, video, resolved);
    let video = palette(&mut context, &video, settings);
    let path = output::bound_path(deliverable, bindings)?;
    let raster = context.canvas;
    let output_args = vec![
        "-r".into(),
        format!(
            "{}/{}",
            raster.frame_rate.numerator, raster.frame_rate.denominator
        ),
        "-s".into(),
        format!("{}x{}", raster.width, raster.height),
        "-c:v".into(),
        "gif".into(),
        "-loop".into(),
        loop_count(settings.playback).to_string(),
        "-f".into(),
        "gif".into(),
        "-t".into(),
        time::seconds(resolved.duration),
    ];
    let inputs = context.input_routes.backend_inputs().to_vec();
    let (filter_graph, filter_contract) = context.filter_graph()?;
    let preparations = context.take_preparations(&inputs);
    let command = BackendCommand {
        preparations,
        inputs,
        filter_graph,
        filter_contract,
        maps: vec![format!("[{video}]")],
        output_args,
        output_path: path.clone(),
    };
    Ok(BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase: BackendPhase::Single,
        product: BackendProduct::AnimatedImage,
        output: BackendOutput::File(path),
        action: BackendAction::Ffmpeg(command),
    })
}

fn palette(context: &mut EmitContext<'_>, input: &str, settings: &GifAnimation) -> String {
    let (frames, source) = context.graph.split(input, "gifsplit");
    let palette = context.graph.filter(
        &[&source],
        "palettegen=stats_mode=diff:reserve_transparent=1",
        "palette",
    );
    context.graph.filter(
        &[&frames, &palette],
        format!(
            "paletteuse=dither={}:diff_mode=rectangle",
            dither(settings.dither)
        ),
        "gif",
    )
}

fn dither(value: GifDither) -> &'static str {
    match value {
        GifDither::Bayer => "bayer",
        GifDither::FloydSteinberg => "floyd_steinberg",
        GifDither::Sierra2 => "sierra2",
        GifDither::None => "none",
    }
}

fn loop_count(value: GifPlayback) -> i32 {
    match value {
        GifPlayback::Once => -1,
        GifPlayback::Forever => 0,
        GifPlayback::Times { count } => i32::from(count) - 1,
    }
}
