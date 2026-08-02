use veac_lang::authoring::lower_document;
use veac_lang::program::compile_source;

const SOURCE: &str = r#"preset text-style title-style {
  font family "Arial"; size 24px; fill #ffffffff;
}
preset text-layout title-layout {
  box-width 320px; box-height 80px; wrap word; overflow clip;
  horizontal-align center; vertical-align middle;
}
preset modifier-stack visual-stack {
  effect stacked-sharpen { type video.sharpen; parameter amount 1.25; }
}
preset effect-pipeline finishing-effects {
  stage effect pipeline-blur { type video.blur; parameter radius 2; }
}
preset color-pipeline finishing-color {
  input-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  working-space { primaries bt709; transfer linear; matrix rgb; range full; }
  output-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  basic {
    exposure 0stops; temperature 6500k; tint 0;
    highlights 0; shadows 0; fade 0%;
  }
}
preset audio-processors voice-chain {
  processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }
}
preset delivery-profile wave-delivery {
  artifact audio-stem voice-stem {
    target file "voice.wav"; source master;
    encode wav {
      sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo;
    }
  }
}
project preset-use-sites {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual picture {
      item title {
        source text {
          content "预设真实使用";
          style { use text-style title-style; }
          layout { use text-layout title-layout; }
        }
        record { at 0s; duration 1s; }
        modifiers { use modifier-stack visual-stack; }
      }
    }
    layer audio voice {
      item narration {
        source generated silence; record { at 0s; duration 1s; }
        modifiers { audio narration-chain { use audio-processors voice-chain; } }
      }
    }
    apply finish {
      scope layer picture; record { at 0s; duration 1s; }
      pipeline {
        use effect-pipeline finishing-effects;
        stage color final-color { use color-pipeline finishing-color; }
      }
      mix {}
    }
  }
  delivery package {
    sequence main;
    use delivery-profile wave-delivery;
  }
}"#;

#[test]
fn every_preset_kind_expands_at_a_real_typed_use_site_and_reaches_ir() {
    let compiled = compile_source(SOURCE).unwrap();
    let expanded = compiled.expanded_source();
    assert!(!expanded.contains("use text-"));
    assert!(!expanded.contains("use modifier-stack"));
    assert!(!expanded.contains("use effect-pipeline"));
    assert!(!expanded.contains("use color-pipeline"));
    assert!(!expanded.contains("use audio-processors"));
    assert!(!expanded.contains("use delivery-profile"));
    for authored in [
        "size 24px",
        "box-width 320px",
        "effect stacked-sharpen",
        "stage effect pipeline-blur",
        "processor limiter",
        "artifact audio-stem voice-stem",
    ] {
        assert!(expanded.contains(authored), "missing expanded {authored}");
    }

    let envelope = lower_document(compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
    let ir = serde_json::to_string(&envelope).unwrap();
    for semantic_id in [
        "stacked-sharpen",
        "pipeline-blur",
        "final-color",
        "limiter",
        "voice-stem",
    ] {
        assert!(ir.contains(semantic_id), "IR omitted {semantic_id}");
    }
}
