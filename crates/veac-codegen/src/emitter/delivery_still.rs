use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AlphaMode, Deliverable, FrameSelection, ImageFormat, StillImage};
use veac_plan::ResolvedRenderPlan;

use super::{
    delivery_image, output, sequence, time, BackendAction, BackendCommand, BackendOutput,
    BackendPhase, BackendProduct, BackendTask, CodegenErrors, EmitContext,
};

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &StillImage,
) -> Result<BackendTask, CodegenErrors> {
    let mut context =
        EmitContext::new_visual(plan, bindings, deliverable, alpha(settings.encoding))?;
    let resolved = context
        .plan
        .sequences
        .iter()
        .find(|value| value.id == context.plan.entry_sequence_id)
        .ok_or_else(|| CodegenErrors::one(super::error::missing_entry(context.plan)))?;
    let video = sequence::build_entry(&mut context, resolved)?;
    let video = sequence::conform_output(&mut context, video, resolved);
    let FrameSelection::Containing { at } = settings.frame;
    let frame = time::containing_frame(at, context.canvas.frame_rate)
        .and_then(|start| start.checked_add(1).map(|end| (start, end)))
        .ok_or_else(|| invalid(deliverable))?;
    let video = context.graph.filter(
        &[&video],
        format!(
            "trim=start_frame={}:end_frame={},setpts=PTS-STARTPTS",
            frame.0, frame.1
        ),
        "still",
    );
    let path = output::bound_path(deliverable, bindings)?;
    let mut output_args = vec!["-frames:v".into(), "1".into()];
    output_args.extend(delivery_image::encoding_arguments(settings.encoding));
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
        product: BackendProduct::StillImage,
        output: BackendOutput::File(path),
        action: BackendAction::Ffmpeg(command),
    })
}

fn alpha(format: ImageFormat) -> AlphaMode {
    match format {
        ImageFormat::Jpeg => AlphaMode::Opaque,
        ImageFormat::Png | ImageFormat::Tiff | ImageFormat::Exr => AlphaMode::Straight,
    }
}

fn invalid(deliverable: &Deliverable) -> CodegenErrors {
    CodegenErrors::one(super::error::diagnostic(
        super::CodegenErrorKind::InvalidPlan,
        "PLAN_STILL_TIME_INVALID",
        Some(deliverable.id.to_string()),
        "still-image sample time does not identify an executable timeline frame",
    ))
}
