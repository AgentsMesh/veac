use super::empty_entry;
use crate::program::{check_source, compile_source};
use crate::source_edit::{ExpressionSite, SourceNodeRef};

#[test]
fn check_source_accepts_standalone_modules_and_entries() {
    assert!(check_source("brand.veac", "module { export const time d = 1s; }").is_ok());
    assert!(check_source("main.veac", &empty_entry("")).is_ok());
    assert_eq!(
        check_source("bad.veac", "module { nonsense; }")
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_DECLARATION"
    );
}

#[test]
fn expression_index_ranges_exclude_surrounding_whitespace_exactly() {
    let source = empty_entry("const time duration =  \n\t 1s + 500ms \t ;");
    let compiled = compile_source(&source).unwrap();
    let index = compiled.source_index().unwrap();
    let target = SourceNodeRef::constant("main.veac", "duration");
    let indexed = index
        .expression(&target, &ExpressionSite::ConstantValue)
        .unwrap();
    assert_eq!(indexed.source, "1s + 500ms");
    assert_eq!(
        &source[indexed.range.start..indexed.range.end],
        "1s + 500ms"
    );
}

#[test]
fn block_comments_and_negative_literals_work_in_the_surface_parser() {
    let source = empty_entry("/* declaration comment */ const scalar offset = -1;");
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("project test"));
}

#[test]
fn modules_reject_instances_and_entries_reject_duplicate_instance_ids() {
    let module = r#"module {
  instance sequence sample from card {}
}"#;
    assert_eq!(
        check_source("invalid.veac", module).unwrap_err().as_slice()[0].code,
        "PROGRAM_MODULE_INSTANCE"
    );
    let duplicate = r#"component sequence card { body {} }
instance sequence same from card {}
instance sequence same from card {}"#;
    assert_eq!(
        compile_source(&empty_entry(duplicate))
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_DUPLICATE_INSTANCE"
    );
}
