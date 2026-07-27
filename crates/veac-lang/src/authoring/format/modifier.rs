use crate::authoring::{
    ApplyStageDecl, EffectParameterValue, LayoutModifierDecl, ModifierDecl, PlacementDecl,
    TransformModifierDecl,
};

use super::parameter::{point, rect, scalar, vector};
use super::value::quoted;
use super::writer::Writer;

pub(super) fn modifier(writer: &mut Writer, value: &ModifierDecl) {
    match value {
        ModifierDecl::Layout(value) => writer
            .block(format!("layout {}", value.id.value), |writer| {
                layout(writer, value)
            }),
        ModifierDecl::Transform(value) => writer
            .block(format!("transform {}", value.id.value), |writer| {
                transform(writer, value)
            }),
        ModifierDecl::Composite(value) => {
            writer.block(format!("composite {}", value.id.value), |writer| {
                if let Some(value) = &value.opacity {
                    scalar(writer, "opacity", value);
                }
                if let Some(value) = &value.z_index {
                    writer.line(format!("z-index {};", value.raw));
                }
                if let Some(value) = &value.blend {
                    writer.line(format!("blend {};", value.value));
                }
            })
        }
        ModifierDecl::Mask(value) => super::modifier_mask::mask(writer, value),
        ModifierDecl::Audio(value) => super::modifier_audio::audio(writer, value),
        ModifierDecl::Color(value) => super::modifier_color::color(writer, value),
        ModifierDecl::Effect(value) => writer
            .block(format!("effect {}", value.id.value), |writer| {
                effect(writer, value)
            }),
    }
}

pub(super) fn apply_stage(writer: &mut Writer, value: &ApplyStageDecl) {
    match value {
        ApplyStageDecl::Color(value) => super::modifier_color::apply_stage(writer, value),
        ApplyStageDecl::Effect(value) => writer
            .block(format!("stage effect {}", value.id.value), |writer| {
                effect(writer, value)
            }),
    }
}

fn layout(writer: &mut Writer, value: &LayoutModifierDecl) {
    if let Some(placement) = &value.placement {
        match placement {
            PlacementDecl::Anchor { anchor, inset, .. } => {
                writer.block("placement anchor", |writer| {
                    writer.line(format!("at {};", anchor.value));
                    vector(
                        writer,
                        "inset",
                        &crate::authoring::ParameterDecl::Constant(inset.clone()),
                    );
                });
            }
            PlacementDecl::Absolute { position, .. } => {
                writer.block("placement absolute", |writer| {
                    point(
                        writer,
                        "position",
                        &crate::authoring::ParameterDecl::Constant(position.clone()),
                    );
                });
            }
        }
    }
    if let Some(value) = &value.frame {
        writer.block("frame", |writer| {
            writer.line(format!("width {};", value.width.raw));
            writer.line(format!("height {};", value.height.raw));
            writer.line(format!("fit {};", value.fit.value));
        });
    }
}

fn transform(writer: &mut Writer, value: &TransformModifierDecl) {
    if let Some(value) = &value.position {
        point(writer, "position", value);
    }
    if let Some(value) = &value.scale {
        vector(writer, "scale", value);
    }
    if let Some(value) = &value.rotation {
        scalar(writer, "rotation", value);
    }
    if let Some(value) = &value.anchor {
        vector(
            writer,
            "anchor",
            &crate::authoring::ParameterDecl::Constant(value.clone()),
        );
    }
    if let Some(value) = &value.crop {
        rect(writer, "crop", value);
    }
    if let Some(value) = &value.flip_horizontal {
        writer.line(format!("flip-horizontal {};", value.value));
    }
    if let Some(value) = &value.flip_vertical {
        writer.line(format!("flip-vertical {};", value.value));
    }
}

fn effect(writer: &mut Writer, value: &crate::authoring::EffectModifierDecl) {
    writer.line(format!("type {};", value.effect_type.value));
    if let Some(value) = &value.enabled {
        writer.line(format!("enabled {};", value.value));
    }
    if let Some(value) = &value.record {
        writer.block("record", |writer| {
            writer.line(format!("at {};", value.at.raw));
            writer.line(format!("duration {};", value.duration.raw));
        });
    }
    let mut parameters: Vec<_> = value.parameters.iter().collect();
    parameters.sort_by(|left, right| left.name.value.cmp(&right.name.value));
    for parameter in parameters {
        match &parameter.value {
            EffectParameterValue::Number(value) => {
                effect_number(writer, &parameter.name.value, value)
            }
            EffectParameterValue::Boolean(value) => writer.line(format!(
                "parameter {} {};",
                parameter.name.value, value.value
            )),
            EffectParameterValue::Color(value) => writer.line(format!(
                "parameter {} {};",
                parameter.name.value, value.value
            )),
            EffectParameterValue::Text(value) => writer.line(format!(
                "parameter {} {};",
                parameter.name.value,
                quoted(&value.value)
            )),
        }
    }
}

fn effect_number(
    writer: &mut Writer,
    name: &str,
    value: &crate::authoring::ParameterDecl<crate::authoring::NumberLiteral>,
) {
    match value {
        crate::authoring::ParameterDecl::Constant(value) => {
            writer.line(format!("parameter {name} {};", value.raw));
        }
        crate::authoring::ParameterDecl::Curve { keys, .. } => {
            writer.block(format!("parameter {name} curve"), |writer| {
                for key in keys {
                    writer.block(format!("key {}", key.id.value), |writer| {
                        writer.line(format!("at {};", key.at.raw));
                        writer.line(format!("value {};", key.value.raw));
                        super::interpolation::write(writer, &key.interpolation);
                    });
                }
            });
        }
    }
}
