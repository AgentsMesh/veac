use std::collections::BTreeSet;

use crate::program::expression::{referenced_symbols, referenced_value_symbols};

#[test]
fn collects_unique_symbols_in_stable_order() {
    let symbols = referenced_symbols(
        "max(brand.title-size, local_offset) + brand.title-size + -card-duration",
    )
    .unwrap();
    assert_eq!(
        symbols,
        BTreeSet::from([
            "brand.title-size".to_owned(),
            "card-duration".to_owned(),
            "local_offset".to_owned(),
        ])
    );
}

#[test]
fn excludes_literals_and_function_names_and_preserves_parse_errors() {
    assert!(referenced_symbols("clamp(1s, 2s, 3s)").unwrap().is_empty());
    assert_eq!(
        referenced_symbols("subject.").unwrap_err().code(),
        "EXPRESSION_FIELD_NAME"
    );
}

#[test]
fn block_locals_are_lexical_and_branches_are_both_collected() {
    let symbols = referenced_symbols(
        "{ let first = external; let second = first + later; \
         if flag { let external = branch; external + second } \
         else { fallback + second } }",
    )
    .unwrap();
    assert_eq!(
        symbols,
        BTreeSet::from([
            "branch".to_owned(),
            "external".to_owned(),
            "fallback".to_owned(),
            "flag".to_owned(),
            "later".to_owned(),
        ])
    );
}

#[test]
fn binding_name_enters_scope_after_its_initializer() {
    assert_eq!(
        referenced_symbols("{ let value = value + seed; value }").unwrap(),
        BTreeSet::from(["seed".to_owned(), "value".to_owned()])
    );
}

#[test]
fn ranges_collect_start_end_and_step_symbols() {
    assert_eq!(
        referenced_symbols("range_start .. range_end by range_step").unwrap(),
        BTreeSet::from([
            "range_end".to_owned(),
            "range_start".to_owned(),
            "range_step".to_owned(),
        ])
    );
}

#[test]
fn closures_collect_captures_but_not_parameters_or_static_callees() {
    assert_eq!(
        referenced_symbols("fn(value: int) -> int effect pure { helper(value) + outside }")
            .unwrap(),
        BTreeSet::from(["outside".to_owned()])
    );
    assert_eq!(
        referenced_symbols("factory()(outside)").unwrap(),
        BTreeSet::from(["outside".to_owned()])
    );
}

#[test]
fn for_binding_is_local_only_inside_its_body() {
    assert_eq!(
        referenced_symbols(
            "for value in value { map(items, fn(item: int) -> int effect pure { item + value + outside }) }"
        )
        .unwrap(),
        BTreeSet::from(["items".to_owned(), "outside".to_owned(), "value".to_owned(),])
    );
}

#[test]
fn value_aware_collection_distinguishes_constant_callees_from_static_functions() {
    let source = "callback(1) + helper(2)";
    assert!(referenced_symbols(source).unwrap().is_empty());
    assert_eq!(
        referenced_value_symbols(source, &|name| name == "callback").unwrap(),
        BTreeSet::from(["callback".to_owned()])
    );
}

#[test]
fn nominal_temporal_and_mutable_forms_keep_only_external_dependencies() {
    assert_eq!(
        referenced_symbols(
            "match choice { Choice.First => outside, \
             Choice.Pair { value } => value + other, }"
        )
        .unwrap(),
        BTreeSet::from([
            "choice".to_owned(),
            "other".to_owned(),
            "outside".to_owned(),
        ])
    );
    assert_eq!(
        referenced_symbols("animate visual-opacity on clip(owner) { progress + external }")
            .unwrap(),
        BTreeSet::from(["external".to_owned(), "owner".to_owned()])
    );
    assert_eq!(
        referenced_symbols(
            "{ var local = initial; set local = update; \
             local.method(argument) + factory(input).field }"
        )
        .unwrap(),
        BTreeSet::from([
            "argument".to_owned(),
            "initial".to_owned(),
            "input".to_owned(),
            "update".to_owned(),
        ])
    );
}
