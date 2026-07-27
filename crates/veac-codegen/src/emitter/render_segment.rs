use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{Deliverable, OutputFormat, VideoDeliverable};
use veac_plan::ResolvedRenderPlan;

use super::{BackendCommand, BackendInput, CodegenErrors};

pub(super) fn command(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &VideoDeliverable,
) -> Result<BackendCommand, CodegenErrors> {
    let segment = bindings
        .full_render_segment()
        .ok_or_else(|| invalid("full render-segment binding disappeared"))?;
    let contract = segment.contract();
    if plan.output.deliverables.len() != 1
        || contract.deliverable_id() != &deliverable.id
        || contract.has_audio() != settings.audio.is_some()
    {
        return Err(invalid(
            "full render segment does not match the emitted video deliverable",
        ));
    }
    let output_path = super::output::bound_path(deliverable, bindings)?;
    let mut maps = vec!["0:0".to_owned()];
    if contract.has_audio() {
        maps.push("0:1".to_owned());
    }
    let mut output_args = vec!["-c".to_owned(), "copy".to_owned()];
    output_args.extend([
        "-map_metadata".to_owned(),
        "-1".to_owned(),
        "-map_chapters".to_owned(),
        "-1".to_owned(),
        "-t".to_owned(),
        super::time::seconds(contract.range().duration),
        "-f".to_owned(),
        container(settings.container).to_owned(),
    ]);
    if settings.optimize_for_streaming {
        output_args.extend(["-movflags".to_owned(), "+faststart".to_owned()]);
    }
    Ok(BackendCommand {
        inputs: vec![BackendInput {
            path: segment.resource().path().to_owned(),
        }],
        filter_graph: None,
        filter_contract: None,
        maps,
        output_args,
        output_path,
    })
}

fn container(value: OutputFormat) -> &'static str {
    match value {
        OutputFormat::Mp4 => "mp4",
        OutputFormat::Mov => "mov",
        OutputFormat::Mkv => "matroska",
        OutputFormat::Webm => "webm",
        OutputFormat::Mxf => "mxf",
    }
}

fn invalid(message: &str) -> CodegenErrors {
    CodegenErrors::one(super::error::diagnostic(
        super::CodegenErrorKind::InvalidResourceBinding,
        "RENDER_SEGMENT_BINDING_INVALID",
        None,
        message,
    ))
}
