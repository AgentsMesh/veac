use crate::authoring::ColorModifierDecl;
use veac_ir::ColorPipeline;

use super::color_stage;
use super::context::Context;

pub(super) fn lower(ctx: &mut Context, value: &ColorModifierDecl) -> Option<ColorPipeline> {
    Some(ColorPipeline {
        input: value.input.value,
        working: value.working.value,
        output: value.output.value,
        stages: value
            .stages
            .iter()
            .map(|value| color_stage::lower(ctx, value))
            .collect::<Option<Vec<_>>>()?,
    })
}
