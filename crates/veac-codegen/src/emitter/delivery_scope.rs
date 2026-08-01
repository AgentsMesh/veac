use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AlphaMode, Deliverable, ImageFormat, ScopeOutput, VideoScope};
use veac_plan::ResolvedRenderPlan;

use super::{
    output, sequence, time, BackendAction, BackendCommand, BackendOutput, BackendPhase,
    BackendProduct, BackendTask, CodegenErrors, EmitContext,
};

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &ScopeOutput,
) -> Result<BackendTask, CodegenErrors> {
    let mut context = EmitContext::new_visual(plan, bindings, deliverable, AlphaMode::Opaque)?;
    let Some(resolved) = context
        .plan
        .sequences
        .iter()
        .find(|value| value.id == context.plan.entry_sequence_id)
    else {
        return Err(CodegenErrors::one(super::error::missing_entry(
            context.plan,
        )));
    };
    let video = sequence::build_entry(&mut context, resolved)?;
    let video = sequence::conform_output(&mut context, video, resolved);
    let Some(frame) = time::containing_frame(settings.at, context.canvas.frame_rate) else {
        return Err(CodegenErrors::one(super::error::diagnostic(
            super::CodegenErrorKind::InvalidPlan,
            "PLAN_SCOPE_OUTPUT_INVALID",
            Some(deliverable.id.to_string()),
            "scope timestamp cannot be represented as an output frame",
        )));
    };
    let Some(end_frame) = frame.checked_add(1) else {
        return Err(CodegenErrors::one(super::error::diagnostic(
            super::CodegenErrorKind::InvalidPlan,
            "PLAN_SCOPE_OUTPUT_INVALID",
            Some(deliverable.id.to_string()),
            "scope frame index exceeds the backend integer domain",
        )));
    };
    let expression = format!(
        "trim=start_frame={frame}:end_frame={end_frame},setpts=PTS-STARTPTS,format=yuv444p,{},scale={}:{}",
        filter(settings.scope),
        settings.width,
        settings.height
    );
    let scoped = context.graph.filter(&[&video], expression, "scopev");
    let inputs = context.input_routes.backend_inputs().to_vec();
    let path = output::bound_path(deliverable, bindings)?;
    let (filter_graph, filter_contract) = context.filter_graph()?;
    let preparations = context.take_preparations(&inputs);
    let command = BackendCommand {
        preparations,
        inputs,
        filter_graph,
        filter_contract,
        maps: vec![format!("[{scoped}]")],
        output_args: image_arguments(settings.format),
        output_path: path.clone(),
    };
    Ok(BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase: BackendPhase::Single,
        product: product(settings.scope),
        output: BackendOutput::File(path),
        action: BackendAction::Ffmpeg(command),
    })
}

fn product(scope: VideoScope) -> BackendProduct {
    match scope {
        VideoScope::Waveform => BackendProduct::VideoWaveform,
        VideoScope::Vectorscope => BackendProduct::Vectorscope,
        VideoScope::Histogram => BackendProduct::Histogram,
    }
}

fn filter(scope: VideoScope) -> &'static str {
    match scope {
        VideoScope::Waveform => "waveform=mode=column:components=7:display=overlay",
        VideoScope::Vectorscope => "vectorscope=mode=color4:graticule=color",
        VideoScope::Histogram => "histogram=display_mode=overlay:level_height=256",
    }
}

fn image_arguments(format: ImageFormat) -> Vec<String> {
    let mut arguments = vec!["-frames:v".to_owned(), "1".to_owned()];
    arguments.extend(super::delivery_image::encoding_arguments(format));
    arguments
}
