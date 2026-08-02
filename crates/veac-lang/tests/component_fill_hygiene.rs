use veac_lang::authoring::SourceDecl;
use veac_lang::program::compile_source;

#[test]
fn nested_fill_references_resolve_in_the_parent_instance_scope() {
    let compiled = compile_source(SOURCE).unwrap();
    let consumer_id = hygienic("root", &["consumer", "wrapped"]);
    let consumer = compiled
        .document()
        .project
        .sequences
        .iter()
        .find(|sequence| sequence.id.value == consumer_id)
        .unwrap();
    let SourceDecl::Sequence { sequence, .. } = &consumer.layers[0].items[0].source else {
        panic!("consumer fill must remain a sequence source");
    };
    assert_eq!(sequence.id.value, hygienic("root", &["surface"]));
    veac_ir::validate(&veac_lang::authoring::lower_document(compiled.document()).unwrap()).unwrap();
}

fn hygienic(root: &str, locals: &[&str]) -> String {
    locals.iter().fold(
        format!("veac-h-{}-{root}", root.len()),
        |mut value, local| {
            value.push_str(&format!("-{}-{local}", local.len()));
            value
        },
    )
}

const SOURCE: &str = r#"component sequence surface {
  body { layer visual @surface-layer { item @surface-item {
    source generated solid { color #176b87ff; }
    record { at 0s; duration 1s; }
  } } }
}
component sequence wrapper {
  slot visual content;
  body { layer visual @wrapper-layer { item @wrapper-item {
    source slot content; record { at 0s; duration 1s; }
  } } }
}
component sequence forwarder {
  slot visual content;
  instance sequence @wrapped from wrapper {
    fill content { source slot content; }
  }
  body { layer visual @forward-layer { item @forward-item {
    source sequence sequence @wrapped; record { at 0s; duration 1s; }
  } } }
}
component sequence shell {
  instance sequence @surface from surface {}
  instance sequence @consumer from forwarder {
    fill content { source sequence sequence @surface; }
  }
  body { layer visual @shell-layer { item @shell-item {
    source sequence sequence @consumer; record { at 0s; duration 1s; }
  } } }
}
instance sequence root from shell {}
project nested-fill-scope {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {}
}"#;

#[test]
fn entry_fills_cannot_capture_callee_private_local_ids() {
    let error = compile_source(ENTRY_CAPTURE).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_FILL_LOCAL_SCOPE");
}

#[test]
fn entry_fill_text_and_comments_may_contain_local_id_spelling() {
    compile_source(ENTRY_LITERAL).unwrap();
}

const ENTRY_CAPTURE: &str = r#"component sequence surface { body {} }
component sequence host {
  slot sequence content;
  instance sequence @private from surface {}
  body { layer visual @layer { item @item {
    source slot content; record { at 0s; duration 1s; }
  } } }
}
instance sequence root from host {
  fill content { source sequence sequence @private; }
}
project entry-fill-scope {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main; sequence main {}
}"#;

const ENTRY_LITERAL: &str = r#"component sequence surface { body {} }
component sequence host {
  slot text content;
  instance sequence @private from surface {}
  body { layer visual @layer { item @item {
    source slot content; record { at 0s; duration 1s; }
  } } }
}
instance sequence root from host {
  fill content { /* @comment */ source text {
    content "@private";
    style { font family "Arial"; size 16px; fill #ffffffff; }
    layout {
      box-width 100px; box-height 40px; wrap word; overflow clip;
      horizontal-align center; vertical-align middle;
    }
  } }
}
project entry-fill-literal {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main; sequence main {}
}"#;
