use super::{empty_entry, entry, error_code};
use crate::program::compile_source;

const CARD_BODY: &str = r#"body {
    layer visual @graphics {
      item @card {
        source generated solid { color #112233ff; }
        record { at 0s; duration ${duration}; }
      }
    }
  }"#;

#[test]
fn parameter_defaults_can_reference_later_parameters() {
    let declarations = format!(
        r#"component sequence card {{
  param time duration default base + 1s;
  param time base default 1s;
  {CARD_BODY}
}}
instance sequence card-one from card {{}}"#
    );
    let source = entry(
        &declarations,
        r#"sequence main { layer visual content { item nested {
    source sequence sequence card-one; record { at 0s; duration 2s; }
  } } }"#,
    );
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("duration 2s;"));
}

#[test]
fn parameter_cycles_fail_even_when_the_component_is_unused() {
    let declaration = format!(
        r#"component sequence card {{
  param time first default second;
  param time second default first;
  {CARD_BODY}
}}"#
    );
    assert_eq!(
        error_code(&empty_entry(&declaration)),
        "PROGRAM_PARAMETER_CYCLE"
    );
}

#[test]
fn parameter_names_shadow_same_named_captured_constants() {
    let declaration = format!(
        r#"const time duration = 9s;
component sequence card {{
  param time duration default duration;
  {CARD_BODY}
}}"#
    );
    assert_eq!(
        error_code(&empty_entry(&declaration)),
        "PROGRAM_PARAMETER_CYCLE"
    );
}

#[test]
fn unused_component_bodies_are_definition_checked() {
    let declaration = r#"component sequence broken {
  body { layer mystery impossible {} }
}"#;
    assert_eq!(
        error_code(&empty_entry(declaration)),
        "PROGRAM_COMPONENT_DEFINITION"
    );
}

#[test]
fn slot_kinds_are_checked_before_core_parsing() {
    let declaration = r#"component sequence title {
  slot text copy;
  body { layer visual graphics { item @copy {
    source slot copy; record { at 0s; duration 1s; }
  } } }
}
instance sequence title-one from title {
  fill copy { source generated transparent; }
}"#;
    assert_eq!(
        error_code(&empty_entry(declaration)),
        "PROGRAM_SLOT_KIND_MISMATCH"
    );
}

#[test]
fn multiple_instances_are_hygienic_and_keep_absolute_provenance() {
    let declarations = format!(
        r#"component sequence card {{ param time duration default 1s; {CARD_BODY} }}
instance sequence first from card {{}}
instance sequence second from card {{}}"#
    );
    let source = entry(&declarations, "sequence main {}");
    let compiled = compile_source(&source).unwrap();
    assert!(compiled
        .expanded_source()
        .contains("item veac-h-5-first-4-card"));
    assert!(compiled
        .expanded_source()
        .contains("item veac-h-6-second-4-card"));
    let expected = source.find("@card").unwrap();
    let first = compiled.provenance().get_local("first", "card").unwrap();
    let second = compiled.provenance().get_local("second", "card").unwrap();
    assert_eq!(first.span.start, expected);
    assert_eq!(second.span.start, expected);
    assert_eq!(first.path, "main.veac");
}

#[test]
fn duplicate_parameters_and_slots_are_rejected_at_declaration() {
    let parameters = r#"component sequence bad {
  param time value; param time value; body {}
}"#;
    assert_eq!(
        error_code(&empty_entry(parameters)),
        "PROGRAM_DUPLICATE_PARAMETER"
    );
    let slots = r#"component sequence bad {
  slot visual value; slot visual value; body {}
}"#;
    assert_eq!(error_code(&empty_entry(slots)), "PROGRAM_DUPLICATE_SLOT");
}

#[test]
fn definition_validation_has_sentinels_for_every_parameter_type() {
    let declaration = r#"component sequence typed {
  param scalar scalar_value;
  param time time_value;
  param length length_value;
  param percent percent_value;
  param angle angle_value;
  param text text_value;
  param color color_value;
  param bool bool_value;
  param identifier identifier_value;
  body {}
}"#;
    assert!(compile_source(&empty_entry(declaration)).is_ok());
}

#[test]
fn definition_validation_has_sources_for_every_slot_kind() {
    let declaration = r#"component sequence slotted {
  slot video video_value;
  slot audio audio_value;
  slot visual visual_value;
  slot text text_value;
  slot caption caption_value;
  slot sequence sequence_value;
  body {
    layer video video_layer { item @video {
      source slot video_value; record { at 0s; duration 1s; }
    } }
    layer audio audio_layer { item @audio {
      source slot audio_value; record { at 0s; duration 1s; }
    } }
    layer visual visual_layer {
      item @visual { source slot visual_value; record { at 0s; duration 1s; } }
      item @text { source slot text_value; record { at 0s; duration 1s; } }
      item @sequence { source slot sequence_value; record { at 0s; duration 1s; } }
    }
    layer caption caption_layer { item @caption {
      source slot caption_value; record { at 0s; duration 1s; }
    } }
  }
}"#;
    assert!(compile_source(&empty_entry(declaration)).is_ok());
}

#[test]
fn media_slots_validate_the_bound_resource_kind() {
    let declaration = r#"component sequence voice {
  slot audio media;
  body { layer audio sound { item @voice {
    source slot media; record { at 0s; duration 1s; }
  } } }
}
instance sequence voice-one from voice {
  fill media { source media resource picture; }
}"#;
    let body = r#"resource video picture {
    locator local { path "picture.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {}"#;
    assert_eq!(
        error_code(&entry(declaration, body)),
        "PROGRAM_SLOT_RESOURCE_KIND"
    );
}
