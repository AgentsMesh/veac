use std::collections::BTreeSet;

use crate::authoring::{format_document, lower_document, parse};
use veac_ir::{EffectDomain, ParameterSpec, ParameterType, ParameterValue, ParameterValueKind};

use super::project;

#[test]
fn registry_parameters_are_closed_across_parse_format_and_lower() {
    let mut registry = BTreeSet::new();
    let mut v2 = BTreeSet::new();
    for effect in veac_ir::built_in_effects() {
        for parameter in effect.parameters {
            let constant = registry_kind(*parameter, false);
            registry.insert(constant);
            v2.insert(verify(effect.effect_type, *parameter, false));
            if parameter.supports_curve {
                registry.insert(registry_kind(*parameter, true));
                v2.insert(verify(effect.effect_type, *parameter, true));
            }
        }
    }
    let canonical: BTreeSet<_> = ParameterValueKind::ALL.into_iter().collect();
    assert_eq!(registry, canonical);
    assert_eq!(v2, canonical);
}

#[test]
fn quoted_effect_parameters_are_not_a_v2_escape_hatch() {
    let error = parse(&document(
        "video.blur",
        "parameter radius \"not a number\";",
    ))
    .unwrap_err();
    assert!(error
        .as_slice()
        .iter()
        .any(|diagnostic| diagnostic.code == "AUTHORING_EFFECT_PARAMETER"));
}

fn verify(effect_type: &str, parameter: ParameterSpec, curve: bool) -> ParameterValueKind {
    let source = document(effect_type, &declaration(parameter, curve));
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{effect_type}: {error}"));
    let authored = authored_kind(&parsed);
    let formatted = format_document(&parsed);
    let reparsed = parse(&formatted).unwrap_or_else(|error| panic!("{effect_type}: {error}"));
    assert_eq!(format_document(&reparsed), formatted);
    let envelope =
        lower_document(&reparsed).unwrap_or_else(|error| panic!("{effect_type}: {error}"));
    let value =
        &envelope.project.sequences[0].tracks[0].clips[0].effects[0].parameters[parameter.name];
    assert_eq!(authored, value.kind());
    assert!(matches!(
        (parameter.value_type, curve, value),
        (ParameterType::Number, false, ParameterValue::Number { .. })
            | (
                ParameterType::Number,
                true,
                ParameterValue::NumberCurve { .. }
            )
            | (
                ParameterType::Boolean,
                false,
                ParameterValue::Boolean { .. }
            )
            | (ParameterType::Color, false, ParameterValue::Color { .. })
    ));
    authored
}

fn authored_kind(document: &crate::authoring::Document) -> ParameterValueKind {
    let modifier = &document.project.sequences[0].layers[0].items[0].modifiers[0];
    let crate::authoring::ModifierDecl::Effect(effect) = modifier else {
        panic!("effect modifier expected")
    };
    effect.parameters[0].value.kind()
}

fn registry_kind(parameter: ParameterSpec, curve: bool) -> ParameterValueKind {
    match (parameter.value_type, curve) {
        (ParameterType::Number, false) => ParameterValueKind::Number,
        (ParameterType::Number, true) if parameter.supports_curve => {
            ParameterValueKind::NumberCurve
        }
        (ParameterType::Boolean, false) => ParameterValueKind::Boolean,
        (ParameterType::Color, false) => ParameterValueKind::Color,
        _ => panic!("registry advertised an invalid parameter mode"),
    }
}

fn document(effect_type: &str, parameter: &str) -> String {
    let (layer, source) = match veac_ir::effect_domain(effect_type) {
        Some(EffectDomain::Video) => ("visual", "source generated transparent;"),
        Some(EffectDomain::Audio) => ("audio", "source generated silence;"),
        None => panic!("registered effect must have a domain"),
    };
    project(&format!(
        r#"sequence main {{
  layer {layer} effects {{
    item subject {{
      {source}
      record {{ at 0s; duration 2s; }}
      modifiers {{
        effect typed {{ type {effect_type}; {parameter} }}
      }}
    }}
  }}
}}"#
    ))
}

fn declaration(parameter: ParameterSpec, curve: bool) -> String {
    let name = parameter.name;
    match parameter.value_type {
        ParameterType::Number if curve => {
            let value = number(parameter);
            format!(
                "parameter {name} curve {{ \
                 key start {{ at 0s; value {value}; interpolation linear; }} \
                 key end {{ at 1s; value {value}; interpolation linear; }} }}"
            )
        }
        ParameterType::Number => format!("parameter {name} {};", number(parameter)),
        ParameterType::Boolean => format!("parameter {name} true;"),
        ParameterType::Color => format!("parameter {name} #12345678;"),
    }
}

fn number(parameter: ParameterSpec) -> f64 {
    match (parameter.minimum, parameter.maximum) {
        (Some(minimum), Some(maximum)) => minimum + (maximum - minimum) / 2.0,
        (Some(minimum), None) => minimum,
        (None, Some(maximum)) => maximum,
        (None, None) => 0.0,
    }
}
