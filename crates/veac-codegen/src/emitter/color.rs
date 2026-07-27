use veac_plan::ResolvedColorPipeline;

use super::{color_space, color_stage, process_owner::ProcessOwner, CodegenErrors, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    input: String,
    pipeline: Option<&ResolvedColorPipeline>,
) -> Result<String, CodegenErrors> {
    let Some(pipeline) = pipeline else {
        return Ok(input);
    };
    let mut label = color_space::convert(context, input, pipeline.input, pipeline.working);
    for stage in &pipeline.stages {
        label = color_stage::apply(context, owner, label, stage)?;
    }
    label = color_space::convert(context, label, pipeline.working, pipeline.output);
    Ok(color_space::tag(context, label, pipeline.output))
}
