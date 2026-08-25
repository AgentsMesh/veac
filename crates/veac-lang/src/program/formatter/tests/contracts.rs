use std::fs;

use tempfile::tempdir;

use super::super::*;

#[test]
fn messy_module_has_a_stable_golden_layout() {
    let source = "module{export const time duration=200ms;export fn rate(value:time)->time{value}}";
    let expected = r#"module {
  export const time duration = 200ms;
  export fn rate(value: time) -> time {
    value
  }
}
"#;
    assert_eq!(format_source(source).unwrap(), expected);
}

#[test]
fn generic_types_and_method_names_remain_glued() {
    let source = r#"module {
  export fn keep(values: list<time>) -> list<time> { values }
  export fn labels() -> map<text, int> { #{"answer": 42} }
  export fn offset(value: scalar) -> scalar { value + (1.0-2.0) + -1.0 }
}
"#;
    let formatted = format_source(source).unwrap_or_else(|error| {
        let tokens = lexer::lex("main.veac", source).unwrap();
        panic!("{error:?}\n{}", layout::format(source, &tokens));
    });
    assert!(formatted.contains("list<time>"));
    assert!(formatted.contains("map<text, int>"));
    assert!(formatted.contains("value + (1.0 - 2.0) + -1.0"));
    assert!(formatted.contains("#{\n"));

    let lexical = "left=>right value.with_snake #{key}";
    let tokens = lexer::lex("tokens.veac", lexical).unwrap();
    let formatted = layout::format(lexical, &tokens);
    let reparsed = lexer::lex("tokens.veac", &formatted).unwrap();
    assert!(same_tokens(&tokens, &reparsed));
    assert!(formatted.contains("=>"));
    assert!(formatted.contains(".with_snake"));
    assert!(formatted.contains("#{"));
}

#[test]
fn named_call_labels_format_canonically_and_idempotently() {
    let source = "module{export fn pair(first:int,second:int)->int{first*10+second}export const int result=pair(second:2,first:1);}";
    let once = format_source(source).unwrap();
    assert!(once.contains("pair(second: 2, first: 1)"));
    assert_eq!(format_source(&once).unwrap(), once);
}

#[test]
fn parameter_defaults_format_canonically_and_idempotently() {
    let source =
        "module{export fn card(title:text,duration:time=3s,color:color=#202124ff)->time{duration}}";
    let once = format_source(source).unwrap();
    assert!(once.contains("title: text, duration: time = 3s, color: color = #202124ff"));
    assert_eq!(format_source(&once).unwrap(), once);
}

#[test]
fn standalone_module_imports_are_resolved_before_and_after_formatting() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("base.veac"),
        "module { export const time duration = 100ms; }",
    )
    .unwrap();
    let root = temp.path().join("timing.veac");
    fs::write(
        &root,
        "module{import \"./base.veac\" as base;export const time duration=base.duration;}",
    )
    .unwrap();
    let formatted = format_path(&root).unwrap();
    assert!(
        formatted.contains("import \"./base.veac\" as base;"),
        "{formatted}"
    );
    assert_eq!(
        format_source_with_loader(
            LoadedSource {
                id: "timing.veac".into(),
                source: formatted.clone(),
            },
            &FileSystemLoader::for_entry(&root).unwrap().0,
        )
        .unwrap(),
        formatted
    );
}

#[test]
fn invalid_syntax_semantics_and_imports_fail_closed() {
    assert!(format_source("module { export fn broken(").is_err());
    assert!(format_source("module { export const time value = \"wrong\"; }").is_err());
    let missing = format_source(
        "module { import \"./missing.veac\" as missing; export const time value = 1s; }",
    )
    .unwrap_err();
    assert_eq!(missing.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
}
