use super::{empty_entry, entry, error_code};
use crate::program::compile_source;

#[test]
fn length_framing_preserves_canonical_instance_and_local_names() {
    let declarations = r#"component sequence left {
  body { layer visual @c {} }
}
component sequence right {
  body { layer visual @b-c {} }
}
instance sequence a-b from left {}
instance sequence a from right {}"#;
    let compiled = compile_source(&entry(declarations, "sequence main {}")).unwrap();
    let source = compiled.expanded_source();
    assert!(source.contains("layer visual veac-h-3-a-b-1-c"));
    assert!(source.contains("layer visual veac-h-1-a-3-b-c"));
}

#[test]
fn an_explicit_id_cannot_claim_a_generated_id() {
    let source = entry(
        r#"component sequence card {
  body { layer visual @graphics {} }
}
instance sequence shot from card {}"#,
        "sequence main { layer visual veac-h-4-shot-8-graphics {} }",
    );
    assert_eq!(error_code(&source), "PROGRAM_HYGIENIC_ID_COLLISION");
}

#[test]
fn provenance_selects_the_declaration_instead_of_local_references() {
    let declarations = r#"component sequence card {
  body {
    apply @before {
      scope layer @graphics;
      record { at 0s; duration 1s; }
      pipeline { stage effect @before-blur { type video.blur; } }
      mix {}
    }
    layer visual @graphics {
      item @picture {
        source generated transparent;
        record { at 0s; duration 1s; }
      }
    }
    apply @after {
      scope layer @graphics;
      record { at 0s; duration 1s; }
      pipeline { stage effect @after-blur { type video.blur; } }
      mix {}
    }
  }
}
instance sequence shot from card {}"#;
    let source = empty_entry(declarations);
    let compiled = compile_source(&source).unwrap();
    let origin = compiled.provenance().get_local("shot", "graphics").unwrap();
    let declaration = source.find("layer visual @graphics").unwrap() + "layer visual ".len();
    assert_eq!(origin.span.start, declaration);
    assert_eq!(&source[origin.span.start..origin.span.end], "@graphics");
    assert_eq!(origin.path, "main.veac");
}

#[test]
fn duplicate_local_declarations_fail_before_provenance_can_overwrite() {
    let source = empty_entry(
        r#"component sequence card {
  body { layer visual @graphics {} layer visual @graphics {} }
}
instance sequence shot from card {}"#,
    );
    assert_eq!(error_code(&source), "PROGRAM_HYGIENIC_ID_COLLISION");
}
