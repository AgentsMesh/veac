use veac_lang::program::compile_source;

#[test]
fn preset_use_after_a_stage_block_is_expanded() {
    let compiled = compile_source(SOURCE).unwrap();
    let expanded = compiled.expanded_source();
    assert!(!expanded.contains("use effect-pipeline base-effects"));
    assert!(expanded.contains("stage effect base-blur"));
    assert!(expanded.contains("stage effect local-sharpen"));
    let canonical = veac_lang::authoring::lower_document(compiled.document()).unwrap();
    veac_ir::validate(&canonical).unwrap();
}

const SOURCE: &str = r#"preset effect-pipeline base-effects {
  stage effect base-blur { type video.blur; parameter radius 2; }
}
preset effect-pipeline combined-effects {
  stage effect local-sharpen { type video.sharpen; parameter amount 1.2; }
  use effect-pipeline base-effects;
}
project preset-statement-boundary {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual picture { item sample {
      source generated solid { color #224466ff; }
      record { at 0s; duration 1s; }
    } }
    apply finish {
      scope layer picture; record { at 0s; duration 1s; }
      pipeline { use effect-pipeline combined-effects; }
      mix {}
    }
  }
}"#;
