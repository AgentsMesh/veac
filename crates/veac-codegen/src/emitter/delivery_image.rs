use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AlphaMode, Deliverable, ImageFormat, ImageSequenceOutput};
use veac_plan::ResolvedRenderPlan;

use super::{output, sequence, time, BackendCommand, CodegenErrors, EmitContext};

pub(super) fn command(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &ImageSequenceOutput,
) -> Result<BackendCommand, CodegenErrors> {
    let alpha = match settings.format {
        ImageFormat::Png | ImageFormat::Tiff | ImageFormat::Exr => AlphaMode::Straight,
        ImageFormat::Jpeg => AlphaMode::Opaque,
    };
    let mut context = EmitContext::new_visual(plan, bindings, deliverable, alpha)?;
    let Some(resolved) = context
        .plan
        .sequences
        .iter()
        .find(|sequence| sequence.id == context.plan.entry_sequence_id)
    else {
        return Err(CodegenErrors::one(super::error::missing_entry(
            context.plan,
        )));
    };
    let video = sequence::build_entry(&mut context, resolved)?;
    let video = sequence::conform_output(&mut context, video, resolved);
    let raster = context.canvas;
    let inputs = context.input_routes.backend_inputs().to_vec();
    let output_path = output::bound_path(deliverable, bindings)?;
    let mut output_args = vec![
        "-r".to_owned(),
        format!(
            "{}/{}",
            raster.frame_rate.numerator, raster.frame_rate.denominator
        ),
        "-s".to_owned(),
        format!("{}x{}", raster.width, raster.height),
        "-start_number".to_owned(),
        settings.start_number.to_string(),
    ];
    output_args.extend(encoding_arguments(settings.format));
    output_args.extend(["-t".to_owned(), time::seconds(resolved.duration)]);
    let (filter_graph, filter_contract) = context.filter_graph()?;
    let preparations = context.take_preparations(&inputs);
    Ok(BackendCommand {
        preparations,
        inputs,
        filter_graph,
        filter_contract,
        maps: vec![format!("[{video}]")],
        output_args,
        output_path,
    })
}

pub(super) fn encoding_arguments(format: ImageFormat) -> Vec<String> {
    match format {
        ImageFormat::Png => vec![
            "-c:v".to_owned(),
            "png".to_owned(),
            "-pix_fmt".to_owned(),
            "rgba".to_owned(),
            "-f".to_owned(),
            "image2".to_owned(),
        ],
        ImageFormat::Jpeg => vec![
            "-c:v".to_owned(),
            "mjpeg".to_owned(),
            "-pix_fmt".to_owned(),
            "yuvj420p".to_owned(),
            "-q:v".to_owned(),
            "2".to_owned(),
            "-f".to_owned(),
            "image2".to_owned(),
        ],
        ImageFormat::Tiff => vec![
            "-c:v".to_owned(),
            "tiff".to_owned(),
            "-pix_fmt".to_owned(),
            "rgba64le".to_owned(),
            "-compression_algo".to_owned(),
            "deflate".to_owned(),
            "-f".to_owned(),
            "image2".to_owned(),
        ],
        ImageFormat::Exr => vec![
            "-c:v".to_owned(),
            "exr".to_owned(),
            "-pix_fmt".to_owned(),
            "gbrapf32le".to_owned(),
            "-compression".to_owned(),
            "zip16".to_owned(),
            "-format".to_owned(),
            "half".to_owned(),
            "-f".to_owned(),
            "image2".to_owned(),
        ],
    }
}
