use veac_ir::{Animatable, ColorStage, EditOperation, Effect, VisualProperty};

use crate::ProviderOutput;

pub(super) fn pipeline(source: &ProviderOutput, operation: &EditOperation) -> bool {
    let ProviderOutput::ColorMatch(result) = source else {
        return false;
    };
    let EditOperation::SetVisualProperty {
        property: VisualProperty::ColorPipeline(Some(pipeline)),
        ..
    } = operation
    else {
        return false;
    };
    let Some((basic, preceding)) = pipeline.stages.split_last() else {
        return false;
    };
    let Some(matrix) = preceding.last() else {
        return false;
    };
    matches!(
        (matrix, basic),
        (
            ColorStage::Matrix { adjustment: matrix },
            ColorStage::Basic { adjustment: basic },
        ) if matrix.matrix == result.adjustment.matrix
            && matrix.offset == result.adjustment.offset
            && basic.exposure_stops == result.adjustment.exposure_stops
            && basic.temperature_kelvin == result.adjustment.temperature_kelvin
            && basic.tint == result.adjustment.tint
            && basic.highlights == 0.0
            && basic.shadows == 0.0
            && basic.fade == 0.0
    )
}

pub(super) fn effect(source: &ProviderOutput, operation: &EditOperation) -> bool {
    let ProviderOutput::ColorMatch(result) = source else {
        return false;
    };
    let EditOperation::AddEffect { effect, .. } = operation else {
        return false;
    };
    matches!(
        &effect.effect,
        Effect::VideoColorAdjust {
            brightness: Animatable::Constant { value: 0.0 },
            contrast: Animatable::Constant { value: contrast },
            saturation: Animatable::Constant { value: saturation },
        } if *contrast == result.adjustment.contrast
            && *saturation == result.adjustment.saturation
    )
}
