use super::Parser;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::ValueType;
use crate::program::model::{PresetKind, SlotKind};

pub(super) fn value(parser: &mut Parser<'_>) -> Result<ValueType, Diagnostic> {
    let (name, span) = parser.word("value type")?;
    match name.as_str() {
        "scalar" => Ok(ValueType::Scalar),
        "time" => Ok(ValueType::Time),
        "length" => Ok(ValueType::Length),
        "percent" => Ok(ValueType::Percent),
        "angle" => Ok(ValueType::Angle),
        "text" => Ok(ValueType::Text),
        "color" => Ok(ValueType::Color),
        "bool" => Ok(ValueType::Boolean),
        "identifier" => Ok(ValueType::Identifier),
        _ => Err(parser.error("PROGRAM_VALUE_TYPE", "unknown value type", span)),
    }
}

pub(super) fn preset(parser: &mut Parser<'_>) -> Result<PresetKind, Diagnostic> {
    let (name, span) = parser.word("preset kind")?;
    match name.as_str() {
        "text-style" => Ok(PresetKind::TextStyle),
        "text-layout" => Ok(PresetKind::TextLayout),
        "modifier-stack" => Ok(PresetKind::ModifierStack),
        "effect-pipeline" => Ok(PresetKind::EffectPipeline),
        "color-pipeline" => Ok(PresetKind::ColorPipeline),
        "audio-processors" => Ok(PresetKind::AudioProcessors),
        "delivery-profile" => Ok(PresetKind::DeliveryProfile),
        _ => Err(parser.error("PROGRAM_PRESET_KIND", "unknown preset kind", span)),
    }
}

pub(super) fn slot(parser: &mut Parser<'_>) -> Result<SlotKind, Diagnostic> {
    let (name, span) = parser.word("slot kind")?;
    match name.as_str() {
        "video" => Ok(SlotKind::Video),
        "audio" => Ok(SlotKind::Audio),
        "visual" => Ok(SlotKind::Visual),
        "text" => Ok(SlotKind::Text),
        "caption" => Ok(SlotKind::Caption),
        "sequence" => Ok(SlotKind::Sequence),
        _ => Err(parser.error("PROGRAM_SLOT_KIND", "unknown slot kind", span)),
    }
}

pub(crate) fn preset_name(kind: PresetKind) -> &'static str {
    match kind {
        PresetKind::TextStyle => "text-style",
        PresetKind::TextLayout => "text-layout",
        PresetKind::ModifierStack => "modifier-stack",
        PresetKind::EffectPipeline => "effect-pipeline",
        PresetKind::ColorPipeline => "color-pipeline",
        PresetKind::AudioProcessors => "audio-processors",
        PresetKind::DeliveryProfile => "delivery-profile",
    }
}
