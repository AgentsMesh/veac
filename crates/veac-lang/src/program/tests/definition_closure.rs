use super::{empty_entry, error_code};

#[test]
fn project_resources_cannot_be_captured_by_component_definitions() {
    for resource in ["validation-video", "validation-first-video"] {
        let declaration = format!(
            r#"component sequence captured {{
  body {{ layer video picture {{ item sample {{
    source media resource {resource}; record {{ at 0s; duration 1s; }}
  }} }} }}
}}"#
        );
        let project = format!(
            r#"resource video {resource} {{
  locator local {{ path "validation.mp4"; }}
  streams {{ video auto; audio disabled; }}
}}
sequence main {{}}"#
        );
        assert_eq!(
            error_code(&super::entry(&declaration, &project)),
            "PROGRAM_COMPONENT_DEFINITION"
        );
    }
}

#[test]
fn captured_constants_cannot_materialize_validation_identities() {
    let declaration = r#"const identifier hidden = identifier("validation-" + "first-video");
component sequence captured {
  body { layer video picture { item sample {
    source media resource ${hidden}; record { at 0s; duration 1s; }
  } } }
}"#;
    assert_eq!(
        error_code(&empty_entry(declaration)),
        "PROGRAM_COMPONENT_DEFINITION"
    );
}

#[test]
fn synthetic_entry_and_computed_identities_cannot_be_captured() {
    for source in [
        "source sequence sequence validation-first-entry;",
        r#"source media resource ${identifier("validation-" + "first-video")};"#,
    ] {
        let declaration = component_with_source(source);
        assert_eq!(
            error_code(&empty_entry(&declaration)),
            "PROGRAM_COMPONENT_DEFINITION",
            "unexpected diagnostic for {source}"
        );
    }
}

#[test]
fn unsupported_unicode_escape_fails_before_definition_validation() {
    let declaration = component_with_source(
        r#"source media resource ${identifier("validation-first-vide\u006f")};"#,
    );
    assert_eq!(error_code(&empty_entry(&declaration)), "PROGRAM_EXPRESSION");
}

#[test]
fn harmless_text_and_unused_identity_constants_do_not_poison_definitions() {
    let declaration = r#"const identifier unused = identifier("validation-first-video");
component sequence copy {
  body { layer visual words { item label {
    source text {
      content "validation-first-video";
      style { font family "Arial"; size 16px; fill #ffffffff; }
      layout {
        box-width 100px; box-height 40px; wrap word; overflow clip;
        horizontal-align center; vertical-align middle;
      }
    }
    record { at 0s; duration 1s; }
  } } }
}"#;
    assert!(super::super::compile_source(&empty_entry(declaration)).is_ok());
}

fn component_with_source(source: &str) -> String {
    format!(
        r#"component sequence captured {{
  body {{ layer visual content {{ item sample {{
    {source} record {{ at 0s; duration 1s; }}
  }} }} }}
}}"#
    )
}
