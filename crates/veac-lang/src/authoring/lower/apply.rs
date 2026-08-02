use crate::authoring::{ApplyDecl, ApplyMixDecl, ApplyStageDecl, SequenceDecl, StructureDecl};
use veac_ir::{
    Animatable, Apply, ApplyMix, ApplyOperation, ApplyStage, BlendMode, Relation, Sequence,
};

use super::context::Context;
use super::{animation, apply_target, ids, modifier_color, modifier_effect, modifier_mask, value};

pub(super) fn lower_sequence(
    ctx: &mut Context,
    declaration: &SequenceDecl,
    sequence: &mut Sequence,
    relations: &[Relation],
) {
    let applies = declaration
        .structures
        .iter()
        .filter_map(|structure| match structure {
            StructureDecl::Apply(value) => lower(ctx, value, sequence, relations),
            StructureDecl::Relation(_) => None,
        })
        .collect();
    sequence.applies = applies;
}

fn lower(
    ctx: &mut Context,
    declaration: &ApplyDecl,
    sequence: &Sequence,
    relations: &[Relation],
) -> Option<Apply> {
    Some(Apply {
        id: ids::apply(ctx, &declaration.id)?,
        enabled: true,
        record_range: value::range(ctx, &declaration.record.at, &declaration.record.duration)?,
        target: apply_target::resolve(ctx, &declaration.scope, sequence, relations)?,
        stages: declaration
            .pipeline
            .iter()
            .map(|stage| lower_stage(ctx, stage))
            .collect::<Option<Vec<_>>>()?,
        mix: lower_mix(ctx, &declaration.mix)?,
    })
}

fn lower_stage(ctx: &mut Context, declaration: &ApplyStageDecl) -> Option<ApplyStage> {
    let (enabled, active_range, operation) = match declaration {
        ApplyStageDecl::Color(value) => (
            true,
            None,
            ApplyOperation::Color {
                pipeline: modifier_color::lower(ctx, value)?,
            },
        ),
        ApplyStageDecl::Effect(value) => {
            let mut effect = modifier_effect::lower(ctx, value)?;
            let enabled = effect.enabled;
            let active_range = effect.enable_range.take();
            effect.enabled = true;
            (enabled, active_range, ApplyOperation::Effect { effect })
        }
    };
    Some(ApplyStage {
        id: ids::apply_stage(ctx, declaration.id())?,
        enabled,
        active_range,
        operation,
    })
}

fn lower_mix(ctx: &mut Context, declaration: &ApplyMixDecl) -> Option<ApplyMix> {
    let opacity = match &declaration.opacity {
        Some(value) => animation::parameter(ctx, value, value::scale)?,
        None => Animatable::constant(1.0),
    };
    let blend_mode = match &declaration.blend {
        Some(value) => super::modifier_visual::blend(ctx, value)?,
        None => BlendMode::Normal,
    };
    let masks = declaration
        .masks
        .iter()
        .map(|value| modifier_mask::lower(ctx, value))
        .collect::<Option<Vec<_>>>()?;
    Some(ApplyMix {
        opacity,
        blend_mode,
        masks,
    })
}
