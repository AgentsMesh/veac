use std::borrow::Cow;

use super::{
    checked_add, consume, ensure_source, Replacements, MAX_EXPANDED_BYTES, MAX_REPLACEMENTS,
};

#[test]
fn replacement_working_set_accepts_the_exact_limit_and_streams_in_source_order() {
    let source = "aXXbYYc";
    let mut replacements =
        Replacements::with_source("test.veac", Cow::Borrowed(source), [1..3, 4..6], 6).unwrap();
    replacements.push_owned("1".to_owned()).unwrap();
    replacements.push_borrowed("2").unwrap();
    assert_eq!(replacements.finish().unwrap(), "a1b2c");
}

#[test]
fn owned_source_is_reused_without_replacements() {
    let source = "owned".to_owned();
    let pointer = source.as_ptr();
    let replacements = Replacements::with_owned("test.veac", source, [], 5).unwrap();
    let output = replacements.finish().unwrap();
    assert_eq!(output.as_ptr(), pointer);
}

#[test]
fn owned_input_temporary_values_and_output_share_the_limit() {
    let source = "aXXb".to_owned();
    let mut rejected =
        Replacements::with_owned("test.veac", source.clone(), std::iter::once(1..3), 7).unwrap();
    assert!(rejected.push_owned("1".to_owned()).is_err());
    assert!(rejected.values.is_empty());

    let mut accepted =
        Replacements::with_owned("test.veac", source, std::iter::once(1..3), 8).unwrap();
    accepted.push_owned("1".to_owned()).unwrap();
    assert_eq!(accepted.finish().unwrap(), "a1b");
}

#[test]
fn oversized_value_is_rejected_before_it_is_retained() {
    let spans = std::iter::once(1..3);
    let mut replacements =
        Replacements::with_source("test.veac", Cow::Borrowed("aXXb"), spans, 3).unwrap();
    let error = replacements.push_owned("12".to_owned()).unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
    assert!(replacements.values.is_empty());
}

#[test]
fn retained_source_and_arithmetic_overflow_fail_before_allocation() {
    let retained = Replacements::with_source("test.veac", Cow::Borrowed("abc"), [], 2).unwrap_err();
    assert_eq!(retained.code, "PROGRAM_EXPANSION_BUDGET");
    assert!(ensure_source("test.veac", 3, 2).is_err());
    assert!(checked_add("test.veac", usize::MAX, 1, "too large").is_err());
    assert!(consume("test.veac", 0, 1).is_err());
}

#[test]
fn repeated_borrowed_values_are_charged_before_the_plan_retains_them() {
    let source = "xxxxxxxx";
    let spans = (0..source.len()).map(|at| at..at + 1);
    let mut replacements =
        Replacements::with_source("test.veac", Cow::Borrowed(source), spans, 7).unwrap();
    for _ in 0..3 {
        replacements.push_borrowed("ab").unwrap();
    }
    let error = replacements.push_borrowed("ab").unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
    assert_eq!(replacements.values.len(), 3);
}

#[test]
fn replacement_count_fails_before_any_values_are_requested() {
    let source = "x".repeat(MAX_REPLACEMENTS + 1);
    let spans = (0..source.len()).map(|at| at..at + 1);
    let error = Replacements::with_source(
        "test.veac",
        Cow::Borrowed(&source),
        spans,
        MAX_EXPANDED_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
    assert!(error.message.contains("replacement budget"));
}

#[test]
fn invalid_or_incomplete_plans_fail_closed() {
    let overlap = Replacements::with_source(
        "test.veac",
        Cow::Borrowed("abcd"),
        [1..3, 2..4],
        MAX_EXPANDED_BYTES,
    )
    .unwrap_err();
    assert!(overlap
        .message
        .contains("invalid expansion replacement range"));

    let incomplete = Replacements::with_source(
        "test.veac",
        Cow::Borrowed("abc"),
        std::iter::once(1..2),
        MAX_EXPANDED_BYTES,
    )
    .unwrap()
    .finish()
    .unwrap_err();
    assert!(incomplete
        .message
        .contains("invalid expansion replacement range"));
}
