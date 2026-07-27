use crate::authoring::{format_document, lower_document, parse};

pub(super) const PROJECT: &str = r#"project text {
  settings {
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30/1fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual titles {
      item title {
        source text {
          content "Hello VEAC";
          style {
            font family "Inter";
            fallback-font family "Arial";
            size 64px;
            weight bold;
            font-style normal;
            fill #ffffffff;
            tracking 1px;
            line-height 1.2;
            background { color #00000099; padding 16px; }
            outline { color #000000ff; width 2px; }
            shadow {
              color #000000ff; opacity 50%; blur 8px;
              offset { x 2px; y 4px; }
            }
            span {
              start 0; end 5; weight extra-bold;
              font-style italic; fill #ffcc00ff; size 72px;
            }
          }
          layout {
            box-width 1200px; box-height 320px;
            wrap none; overflow visible;
            horizontal-align center; vertical-align middle;
            writing-mode horizontal-tb; orientation upright;
            path {
              point { x 10%; y 50%; }
              point { x 90%; y 50%; }
              start-offset 0px; reverse false; align start;
            }
          }
          animation {
            unit word; stagger 100ms; reveal 100%; opacity 100%;
            highlight { fill #ff3366ff; progress 50%; }
            transform {
              position { x 0px; y 0px; }
              scale { x 100%; y 100%; }
              rotation 0deg;
            }
          }
        }
        record { at 0s; duration 3s; }
      }
    }
  }
}"#;

#[test]
fn text_primitives_parse_format_lower_and_validate() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope =
        lower_document(&document).unwrap_or_else(|error| panic!("{:#?}", error.as_slice()));
    let source = &envelope.project.sequences[0].tracks[0].clips[0].source;
    let veac_ir::ClipSource::Text { style, .. } = source else {
        panic!("text source expected")
    };
    assert_eq!(style.spans.len(), 1);
    assert!(style.path.is_some());
    assert!(style.animation.is_some());
    assert!(style
        .animation
        .as_ref()
        .is_some_and(|animation| animation.highlight.is_some()));
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn old_leaf_text_syntax_is_rejected() {
    let source = PROJECT.replace(
        "source text {\n          content \"Hello VEAC\";",
        "source text \"Hello VEAC\";",
    );
    assert!(parse(&source).is_err());
}

#[test]
fn unknown_text_enum_is_rejected() {
    let source = PROJECT.replace("wrap none;", "wrap sentence;");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_TEXT_FIELD"));
}
