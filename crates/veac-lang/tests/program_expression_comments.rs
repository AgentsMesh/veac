use veac_lang::program::compile_source;

#[test]
fn placeholder_closing_braces_inside_comments_are_ignored() {
    for expression in ["${1s /* ignored } */ + 2s}", "${1s // ignored }\n + 2s}"] {
        let compiled = compile_source(&source(expression)).unwrap();
        let root = compiled
            .document()
            .project
            .sequences
            .iter()
            .find(|sequence| sequence.id.value == "root")
            .unwrap();
        assert_eq!(root.layers[0].items[0].record.duration.raw, "3s");
        let canonical = veac_lang::authoring::lower_document(compiled.document()).unwrap();
        veac_ir::validate(&canonical).unwrap();
    }
}

fn source(expression: &str) -> String {
    format!(
        r#"component sequence card {{
  body {{ layer visual @layer {{ item @item {{
    source generated transparent;
    record {{ at 0s; duration {expression}; }}
  }} }} }}
}}
instance sequence root from card {{}}
project expression-comments {{
  settings {{
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }}
  entry sequence main;
  sequence main {{}}
}}"#
    )
}
