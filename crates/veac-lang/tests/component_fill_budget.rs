use veac_lang::program::compile_source;

#[test]
fn forwarded_slot_fills_share_one_active_frame_memory_budget() {
    let source = forwarding_program(8, 4 * 1024 * 1024);
    let error = compile_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_EXPANSION_BUDGET");
}

#[test]
fn completed_root_instances_release_their_fill_memory() {
    let source = sequential_program(11 * 1024 * 1024);
    let compiled = compile_source(&source).unwrap();
    for id in ["first", "second", "third"] {
        assert!(compiled
            .document()
            .project
            .sequences
            .iter()
            .any(|sequence| sequence.id.value == id));
    }
}

fn forwarding_program(levels: usize, fill_bytes: usize) -> String {
    let mut declarations = r#"component sequence leaf {
  slot visual payload;
  body { layer visual @layer { item @item {
    source slot payload; record { at 0s; duration 1s; }
  } } }
}
"#
    .to_owned();
    let mut child = "leaf".to_owned();
    for level in 0..levels {
        let name = format!("level-{level}");
        declarations.push_str(&format!(
            r#"component sequence {name} {{
  slot visual payload;
  instance sequence @child from {child} {{
    fill payload {{ source slot payload; }}
  }}
  body {{ layer visual @layer {{ item @item {{
    source sequence sequence @child; record {{ at 0s; duration 1s; }}
  }} }} }}
}}
"#
        ));
        child = name;
    }
    let filler = "x".repeat(fill_bytes);
    format!(
        "{declarations}instance sequence root from {child} {{\n  fill payload {{ \
         source generated transparent; /*{filler}*/ }}\n}}\n{}",
        project()
    )
}

fn project() -> &'static str {
    r#"project forwarded-fill-budget {
  settings {
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 8000hz;
  }
  entry sequence main; sequence main {}
}"#
}

fn sequential_program(fill_bytes: usize) -> String {
    let payload = "x".repeat(fill_bytes);
    format!(
        r#"component sequence sink {{ slot visual payload; body {{}} }}
component sequence carrier {{
  instance sequence @sink from sink {{
    fill payload {{ source generated transparent; /*{payload}*/ }}
  }}
  body {{}}
}}
instance sequence first from carrier {{}}
instance sequence second from carrier {{}}
instance sequence third from carrier {{}}
{}"#,
        project()
    )
}
