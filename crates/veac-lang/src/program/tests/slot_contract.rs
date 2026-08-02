use super::{empty_entry, error_code};
use crate::program::compile_source;

fn source(fill: &str) -> String {
    empty_entry(&format!(
        r#"component sequence card {{
  slot visual picture;
  body {{ layer visual content {{ item @picture {{
    source slot picture; record {{ at 0s; duration 1s; }}
  }} }} }}
}}
instance sequence example from card {{
  fill picture {{ {fill} }}
}}"#
    ))
}

#[test]
fn slot_fill_rejects_every_trailing_item_member_and_second_source() {
    for fill in [
        "source generated transparent; modifiers { effect injected { type video.blur; } }",
        "source generated transparent; state { playback disabled; }",
        "source generated transparent; record { at 5s; duration 1s; }",
        "source generated transparent; source generated transparent;",
    ] {
        assert_eq!(error_code(&source(fill)), "PROGRAM_SLOT_SOURCE", "{fill}");
    }
}

#[test]
fn slot_fill_accepts_one_complete_nested_source() {
    let fill = r#"source generated gradient linear {
      from 0% 0%; to 100% 100%;
      stop 0% #112233ff; stop 100% #445566ff;
    }"#;
    let compiled = compile_source(&source(fill)).unwrap();
    assert!(compiled.expanded_source().contains("stop 100% #445566ff"));
}
