use std::collections::BTreeSet;

use crate::program::expression::referenced_symbols;

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
        "EXPRESSION_SYMBOL"
    );
}
