use crate::authoring::{format_document, parse};

use super::project;

#[test]
fn formatter_quotes_text_effect_parameters_from_authored_ast() {
    let source = project(
        r#"sequence main {
  layer visual picture {
    item blank {
      source generated transparent;
      record { at 0s; duration 1s; }
      modifiers {
        effect keyed { type video.luma_key; parameter invert true; }
      }
    }
  }
}"#,
    );
    let mut document = parse(&source).unwrap();
    let effect = document.project.sequences[0].layers[0].items[0]
        .modifiers
        .iter_mut()
        .find_map(|value| match value {
            crate::authoring::ModifierDecl::Effect(value) => Some(value),
            _ => None,
        })
        .unwrap();
    effect.parameters[0].value =
        crate::authoring::EffectParameterValue::Text(crate::authoring::Spanned {
            value: "quoted value".to_owned(),
            span: Default::default(),
        });
    let formatted = format_document(&document);
    assert!(formatted.contains("parameter invert \"quoted value\";"));
}
