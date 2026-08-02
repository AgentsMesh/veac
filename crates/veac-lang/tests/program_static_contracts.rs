use veac_lang::program::compile_source;

fn project(declarations: &str) -> String {
    format!(
        r#"{declarations}
project contracts {{
  settings {{
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }}
  entry sequence main;
  sequence main {{}}
}}"#
    )
}

fn error_code(source: &str) -> &'static str {
    compile_source(source).unwrap_err().as_slice()[0].code
}

#[test]
fn public_compile_rejects_cross_kind_preset_composition() {
    let declarations = r#"preset text-layout empty {}
preset text-style title {
  use text-layout empty;
  font family "Arial"; size 24px; fill #ffffffff;
}"#;
    assert_eq!(
        error_code(&project(declarations)),
        "PROGRAM_PRESET_KIND_MISMATCH"
    );
}

#[test]
fn public_compile_rejects_slot_fill_sibling_injection() {
    let declarations = r#"component sequence card {
  slot visual picture;
  body { layer visual content { item @picture {
    source slot picture; record { at 0s; duration 1s; }
  } } }
}
instance sequence example from card {
  fill picture {
    source generated transparent;
    modifiers { effect injected { type video.blur; } }
  }
}"#;
    assert_eq!(error_code(&project(declarations)), "PROGRAM_SLOT_SOURCE");
}
