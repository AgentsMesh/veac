use super::empty_entry;
use crate::program::compile_source;

#[test]
fn unused_component_runs_lower_validation() {
    let source = empty_entry(
        r#"component sequence broken {
  body { layer visual @content { item @sample {
    source generated transparent;
    record { at 0s; duration 1.5ms; }
  } } }
}"#,
    );
    let errors = compile_source(&source).unwrap_err();
    let error = &errors.as_slice()[0];
    assert_eq!(error.code, "PROGRAM_COMPONENT_DEFINITION");
    assert!(error.message.contains("AUTHORING_LOWER_TIME_PRECISION"));
}

#[test]
fn unused_preset_runs_canonical_validation() {
    let source = empty_entry(
        r#"preset modifier-stack broken {
  effect keyed {
    type video.luma_key;
    parameter threshold 1.5;
  }
}"#,
    );
    let errors = compile_source(&source).unwrap_err();
    let error = &errors.as_slice()[0];
    assert_eq!(error.code, "PROGRAM_PRESET_DEFINITION");
    assert!(error.message.contains("AUTHORING_LOWER_IR_VALIDATION"));
    assert!(error.message.contains("EFFECT_PARAMETER"));
}

#[test]
fn identifier_sentinel_is_valid_for_resource_and_sequence_references() {
    let source = empty_entry(
        r#"component sequence references {
  param identifier source_id;
  body {
    layer video @video { item @media {
      source media resource ${source_id};
      record { at 0s; duration 1s; }
    } }
    layer visual @visual { item @nested {
      source sequence sequence ${source_id};
      record { at 0s; duration 1s; }
    } }
  }
}"#,
    );
    compile_source(&source).unwrap();
}
