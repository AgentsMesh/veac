use super::{empty_entry, entry, error_code};
use crate::program::compile_source;

#[test]
fn placeholders_inside_comments_and_strings_remain_lossless() {
    let body = r#"sequence main {
    // ${missing_line}
    /* ${missing_block} */
    layer visual text { item sample {
      source text {
        content "${missing_string}";
        style { font family "Arial"; size 16px; fill #ffffffff; }
        layout {
          box-width 320px; box-height 80px; wrap word; overflow clip;
          horizontal-align center; vertical-align middle;
        }
      }
      record { at 0s; duration 1s; }
    } }
  }"#;
    let source = entry("", body);
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("${missing_line}"));
    assert!(compiled.expanded_source().contains("${missing_block}"));
    assert!(compiled.expanded_source().contains("${missing_string}"));
}

#[test]
fn expression_closing_braces_inside_strings_do_not_end_placeholders() {
    let source = empty_entry(r#"const text value = "a}b";"#);
    assert!(compile_source(&source).is_ok());
}

#[test]
fn replacement_count_is_bounded() {
    let declarations = "const scalar value = 1;";
    let fields = "order ${value};\n".repeat(16_385);
    let body = format!("sequence main {{ layer visual values {{ {fields} }} }}");
    let source = entry(declarations, &body);
    assert_eq!(error_code(&source), "PROGRAM_EXPANSION_BUDGET");
}
