use veac_lang::authoring::lower_document;
use veac_lang::program::compile_source;

const SOURCE: &str = r#"component sequence six-slot-card {
  slot video video-source;
  slot audio audio-source;
  slot visual visual-source;
  slot text text-source;
  slot caption caption-source;
  slot sequence sequence-source;
  body {
    layer video @video-layer { item @video {
      source slot video-source; record { at 0s; duration 1s; }
    } }
    layer audio @audio-layer { item @audio {
      source slot audio-source; record { at 0s; duration 1s; }
    } }
    layer visual @visual-layer {
      item @visual { source slot visual-source; record { at 0s; duration 1s; } }
      item @text { source slot text-source; record { at 0s; duration 1s; } }
      item @sequence { source slot sequence-source; record { at 0s; duration 1s; } }
    }
    layer caption @caption-layer { item @caption {
      source slot caption-source; record { at 0s; duration 1s; }
    } }
  }
}
instance sequence filled-card from six-slot-card {
  fill video-source { source media resource picture; }
  fill audio-source { source media resource voice; }
  fill visual-source { source generated solid { color #224466ff; } }
  fill text-source { source text {
    content "文本插槽";
    style { font family "Arial"; size 20px; fill #ffffffff; }
    layout {
      box-width 320px; box-height 80px; wrap word; overflow clip;
      horizontal-align center; vertical-align middle;
    }
  } }
  fill caption-source { source caption {
    content "字幕插槽";
    style { font family "Arial"; size 20px; fill #ffffffff; }
    layout {
      box-width 320px; box-height 80px; wrap word; overflow clip;
      horizontal-align center; vertical-align middle;
    }
  } }
  fill sequence-source { source sequence sequence nested; }
}
project slot-use-sites {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  resource video picture {
    locator local { path "picture.mp4"; }
    streams { video auto; audio disabled; }
  }
  resource audio voice {
    locator local { path "voice.wav"; }
    streams { video disabled; audio auto; }
  }
  entry sequence main;
  sequence nested {}
  sequence main { layer visual content { item card {
    source sequence sequence filled-card; record { at 0s; duration 1s; }
  } } }
}"#;

#[test]
fn every_slot_kind_accepts_a_real_caller_fill_and_reaches_ir() {
    let compiled = compile_source(SOURCE).unwrap();
    let expanded = compiled.expanded_source();
    assert!(!expanded.contains("source slot"));
    for fill in [
        "source media resource picture",
        "source media resource voice",
        "color #224466ff",
        "content \"文本插槽\"",
        "content \"字幕插槽\"",
        "source sequence sequence nested",
    ] {
        assert!(expanded.contains(fill), "missing caller fill {fill}");
    }

    let envelope = lower_document(compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
    let ir = serde_json::to_string(&envelope).unwrap();
    for authored in ["picture", "voice", "文本插槽", "字幕插槽", "nested"] {
        assert!(ir.contains(authored), "IR omitted slot value {authored}");
    }
}

#[test]
fn slot_source_after_a_record_block_is_still_a_complete_statement() {
    let reordered = SOURCE.replacen(
        "source slot video-source; record { at 0s; duration 1s; }",
        "record { at 0s; duration 1s; } source slot video-source;",
        1,
    );
    let compiled = compile_source(&reordered).unwrap();
    assert!(!compiled.expanded_source().contains("source slot"));
    let envelope = lower_document(compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
}
