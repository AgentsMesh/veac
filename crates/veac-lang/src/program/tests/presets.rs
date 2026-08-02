use super::{empty_entry, entry, error_code};
use crate::program::compile_source;

fn text_sequence(style: &str, content: &str) -> String {
    format!(
        r#"sequence main {{ layer visual text {{ item title {{
    source text {{
      content "{content}";
      style {{ {style} }}
      layout {{
        box-width 320px; box-height 80px; wrap word; overflow clip;
        horizontal-align center; vertical-align middle;
      }}
    }}
    record {{ at 0s; duration 1s; }}
  }} }} }}"#
    )
}

#[test]
fn presets_support_forward_and_nested_references() {
    let declarations = r#"preset text-style outer { use text-style middle; }
preset text-style middle { use text-style base; }
preset text-style base {
  font family "Arial"; size 24px; fill #ffffffff;
}"#;
    let source = entry(
        declarations,
        &text_sequence("use text-style outer;", "title"),
    );
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("size 24px"));
    assert!(!compiled.expanded_source().contains("use text-style"));
}

#[test]
fn preset_cycles_and_kind_mismatches_are_explicit() {
    let cycle = r#"preset text-style first { use text-style second; }
preset text-style second { use text-style first; }"#;
    assert_eq!(error_code(&empty_entry(cycle)), "PROGRAM_PRESET_CYCLE");
    let mismatch = r#"preset text-layout shared {}
preset text-style title { use text-style shared; }"#;
    assert_eq!(
        error_code(&empty_entry(mismatch)),
        "PROGRAM_PRESET_KIND_MISMATCH"
    );
}

#[test]
fn preset_composition_requires_the_enclosing_kind() {
    let mismatch = r#"preset text-layout empty {}
preset text-style title {
  use text-layout empty;
  font family "Arial"; size 24px; fill #ffffffff;
}"#;
    assert_eq!(
        error_code(&empty_entry(mismatch)),
        "PROGRAM_PRESET_KIND_MISMATCH"
    );
}

#[test]
fn unused_presets_are_definition_checked() {
    let source = empty_entry("preset text-style broken { impossible syntax; }");
    assert_eq!(error_code(&source), "PROGRAM_PRESET_DEFINITION");
}

#[test]
fn use_like_text_in_strings_and_comments_is_not_expanded() {
    let style = r#"// use text-style missing-line;
        /* use text-style missing-block; */
        font family "Arial"; size 16px; fill #ffffffff;"#;
    let source = entry("", &text_sequence(style, "use text-style missing-string;"));
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("missing-string"));
    assert!(compiled.expanded_source().contains("missing-block"));
}

#[test]
fn every_closed_preset_kind_is_definition_checked_in_its_typed_context() {
    let declarations = r#"preset modifier-stack modifiers {
  effect blur { type video.blur; }
}
preset effect-pipeline effects {
  stage effect blur { type video.blur; }
}
preset color-pipeline color {
  input-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  working-space { primaries bt709; transfer linear; matrix rgb; range full; }
  output-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  basic {
    exposure 0stops; temperature 6500k; tint 0;
    highlights 0; shadows 0; fade 0%;
  }
}
preset audio-processors audio {
  processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }
}
preset delivery-profile delivery {
  artifact audio-stem audio {
    target file "validation.wav"; source master;
    encode wav {
      sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo;
    }
  }
}"#;
    compile_source(&empty_entry(declarations)).unwrap();
}
