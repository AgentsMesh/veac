use super::parameter::scalar;
use super::semantic::{boolean, finish, required, take, word};
use super::Parser;
use crate::authoring::{
    EffectModifierDecl, EffectParameterDecl, EffectParameterValue, Identifier, ParameterDecl,
    RecordSpan, SemanticBlock, SemanticEntry, SemanticValue,
};

impl Parser {
    pub(super) fn effect_modifier(
        &mut self,
        id: Identifier,
        mut body: SemanticBlock,
    ) -> Option<EffectModifierDecl> {
        let type_entry = required(self, &mut body, "type", "effect")?;
        let effect_type = word(self, &type_entry, "effect type")?;
        if veac_ir::built_in_effect(&effect_type.value).is_none() {
            self.error(
                "AUTHORING_EFFECT_TYPE",
                format!("unknown effect type `{}`", effect_type.value),
                effect_type.span,
            );
        }
        let enabled = take(self, &mut body, "enabled")
            .and_then(|entry| boolean(self, &entry, "effect enabled"));
        let record = take(self, &mut body, "record").and_then(|entry| self.effect_record(&entry));
        let mut parameters = Vec::new();
        let mut retained = Vec::new();
        for entry in body.entries.drain(..) {
            if entry.name.value == "parameter" {
                if let Some(value) = self.effect_parameter(&entry) {
                    parameters.push(value);
                }
            } else {
                retained.push(entry);
            }
        }
        body.entries = retained;
        duplicate_parameters(self, &parameters);
        validate_parameter_names(self, &effect_type, &parameters);
        let span = id.span.join(body.span);
        finish(self, body, "effect modifier");
        Some(EffectModifierDecl {
            id,
            effect_type,
            enabled,
            record,
            parameters,
            span,
        })
    }

    fn effect_record(&mut self, entry: &SemanticEntry) -> Option<RecordSpan> {
        let mut body = super::semantic::nested(self, entry, "effect record")?;
        let at = required(self, &mut body, "at", "effect record")?;
        let duration = required(self, &mut body, "duration", "effect record")?;
        let value = RecordSpan {
            at: super::semantic::number(self, &at, "effect record at")?,
            duration: super::semantic::number(self, &duration, "effect record duration")?,
            span: entry.span,
        };
        finish(self, body, "effect record");
        Some(value)
    }

    fn effect_parameter(&mut self, entry: &SemanticEntry) -> Option<EffectParameterDecl> {
        let (name, tail) = entry.values.split_first()?;
        let SemanticValue::Identifier(name) = name else {
            self.error(
                "AUTHORING_EFFECT_PARAMETER",
                "parameter requires a name".to_owned(),
                entry.span,
            );
            return None;
        };
        let synthetic = SemanticEntry {
            name: name.clone(),
            values: tail.to_vec(),
            block: entry.block.clone(),
            span: entry.span,
        };
        let value = match tail {
            [SemanticValue::Number(_)] | [SemanticValue::Identifier(_)] => {
                EffectParameterValue::Number(scalar(self, &synthetic)?)
            }
            [SemanticValue::Boolean(value)] if entry.block.is_none() => {
                EffectParameterValue::Boolean(value.clone())
            }
            [SemanticValue::Color(value)] if entry.block.is_none() => {
                EffectParameterValue::Color(value.clone())
            }
            [SemanticValue::String(value)] if entry.block.is_none() => {
                EffectParameterValue::Text(value.clone())
            }
            _ => {
                self.error(
                    "AUTHORING_EFFECT_PARAMETER",
                    "parameter value must be a number, curve, boolean, color, or string".to_owned(),
                    entry.span,
                );
                return None;
            }
        };
        Some(EffectParameterDecl {
            name: name.clone(),
            value,
            span: entry.span,
        })
    }
}

fn duplicate_parameters(parser: &mut Parser, values: &[EffectParameterDecl]) {
    let mut names = std::collections::BTreeSet::new();
    for value in values {
        if !names.insert(value.name.value.as_str()) {
            parser.error(
                "AUTHORING_DUPLICATE_FIELD",
                format!("effect parameter `{}` is repeated", value.name.value),
                value.span,
            );
        }
    }
}

fn validate_parameter_names(
    parser: &mut Parser,
    effect_type: &Identifier,
    values: &[EffectParameterDecl],
) {
    let Some(spec) = veac_ir::built_in_effect(&effect_type.value) else {
        return;
    };
    for value in values {
        let parameter = spec
            .parameters
            .iter()
            .find(|item| item.name == value.name.value);
        if parameter.is_none() {
            parser.error(
                "AUTHORING_EFFECT_PARAMETER_NAME",
                format!(
                    "effect `{}` has no parameter `{}`",
                    effect_type.value, value.name.value
                ),
                value.name.span,
            );
        } else if !parameter.is_some_and(|item| parameter_shape(*item, &value.value)) {
            parser.error(
                "AUTHORING_EFFECT_PARAMETER_TYPE",
                format!(
                    "parameter `{}` has the wrong value type for `{}`",
                    value.name.value, effect_type.value
                ),
                value.span,
            );
        }
    }
}

fn parameter_shape(spec: veac_ir::ParameterSpec, value: &EffectParameterValue) -> bool {
    match (spec.value_type, value) {
        (
            veac_ir::ParameterType::Number,
            EffectParameterValue::Number(ParameterDecl::Constant(_)),
        ) => true,
        (
            veac_ir::ParameterType::Number,
            EffectParameterValue::Number(ParameterDecl::Curve { .. }),
        ) => spec.supports_curve,
        (veac_ir::ParameterType::Boolean, EffectParameterValue::Boolean(_))
        | (veac_ir::ParameterType::Color, EffectParameterValue::Color(_))
        | (veac_ir::ParameterType::Text, EffectParameterValue::Text(_)) => true,
        _ => false,
    }
}
