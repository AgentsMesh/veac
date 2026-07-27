use std::collections::BTreeMap;

use veac_ir::{
    BasicColorAdjustment, ColorPipeline, ColorStage, EditOperation, EffectInstance, ParameterValue,
    Precondition, ProjectEnvelope, RgbMatrixAdjustment, VisualProperty,
};

use super::support::{self, BuiltApplication};
use crate::{ColorMatchApplication, ColorMatchResult, ProposalEvidence, ProviderResult};

pub(super) fn build(
    project: &ProjectEnvelope,
    result: &ColorMatchResult,
    context: &ColorMatchApplication,
) -> ProviderResult<BuiltApplication> {
    let (track, clip) = support::clip(project, &context.clip_id)?;
    let Some(visual) = clip.visual.as_ref() else {
        return support::invalid("color-match target must have visual properties");
    };
    if track.state.locked {
        return support::invalid("color-match target track is locked");
    }
    validate_adjustment(result)?;
    let mut pipeline = visual.color_pipeline.clone().unwrap_or(ColorPipeline {
        input: context.color_space,
        working: context.color_space,
        output: context.color_space,
        stages: Vec::new(),
    });
    pipeline.stages.push(ColorStage::Matrix {
        adjustment: RgbMatrixAdjustment {
            matrix: result.adjustment.matrix,
            offset: result.adjustment.offset,
        },
    });
    pipeline.stages.push(ColorStage::Basic {
        adjustment: BasicColorAdjustment {
            exposure_stops: result.adjustment.exposure_stops,
            temperature_kelvin: result.adjustment.temperature_kelvin,
            tint: result.adjustment.tint,
            highlights: 0.0,
            shadows: 0.0,
            fade: 0.0,
        },
    });
    let color_effect = EffectInstance {
        id: context.effect_id.clone(),
        effect_type: "video.color_adjust".to_owned(),
        enabled: true,
        enable_range: None,
        parameters: BTreeMap::from([
            (
                "brightness".to_owned(),
                ParameterValue::Number { value: 0.0 },
            ),
            (
                "contrast".to_owned(),
                ParameterValue::Number {
                    value: result.adjustment.contrast,
                },
            ),
            (
                "saturation".to_owned(),
                ParameterValue::Number {
                    value: result.adjustment.saturation,
                },
            ),
        ]),
    };
    let operations = vec![
        EditOperation::SetVisualProperty {
            clip_id: context.clip_id.clone(),
            property: VisualProperty::ColorPipeline(Some(pipeline)),
        },
        EditOperation::AddEffect {
            clip_id: context.clip_id.clone(),
            effect: color_effect,
            before_id: None,
            after_id: clip.effects.last().map(|effect| effect.id.clone()),
        },
    ];
    let evidence = vec![
        ProposalEvidence::ColorPipeline {
            operation: crate::OperationBinding::new(0, &operations[0])?,
        },
        ProposalEvidence::ColorAdjustEffect {
            operation: crate::OperationBinding::new(1, &operations[1])?,
        },
    ];
    let mut built = BuiltApplication::new(operations, evidence);
    built.preconditions.push(Precondition::TrackUnlocked {
        track_id: track.id.clone(),
    });
    Ok(built)
}

fn validate_adjustment(result: &ColorMatchResult) -> ProviderResult<()> {
    let value = &result.adjustment;
    if !value
        .matrix
        .iter()
        .all(|component| (-16.0..=16.0).contains(component))
        || !value
            .offset
            .iter()
            .all(|component| (-4.0..=4.0).contains(component))
        || !(-10.0..=10.0).contains(&value.exposure_stops)
        || !(1000.0..=40000.0).contains(&value.temperature_kelvin)
        || !(-1.0..=1.0).contains(&value.tint)
        || !(0.0..=4.0).contains(&value.contrast)
        || !(0.0..=4.0).contains(&value.saturation)
    {
        return support::invalid("color-match adjustment exceeds canonical property ranges");
    }
    Ok(())
}
