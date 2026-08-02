use super::*;

#[test]
fn exact_boundaries_succeed_and_failed_charges_do_not_commit() {
    let value = Value::Text("payload".to_owned());
    let exact = ENTRY_BYTES + "name".len() + value.retained_bytes();
    let mut budget = Budget {
        bytes: 0,
        limit: exact,
    };
    budget
        .value("main.veac", "name", &value, Span::default())
        .unwrap();
    assert_eq!(budget.bytes, exact);

    let error = budget
        .alias("main.veac", "next", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
    assert_eq!(budget.bytes, exact);
}

#[test]
fn aliases_charge_entries_without_recharging_shared_payloads() {
    let value = Value::Text("large-payload".repeat(100));
    let first = ENTRY_BYTES + "source".len() + value.retained_bytes();
    let aliases = ["left.source", "right.source"];
    let total = aliases
        .iter()
        .fold(first, |bytes, name| bytes + ENTRY_BYTES + name.len());
    let mut budget = Budget {
        bytes: 0,
        limit: total,
    };
    budget
        .value("module.veac", "source", &value, Span::default())
        .unwrap();
    for name in aliases {
        budget.alias("main.veac", name, Span::default()).unwrap();
    }
    assert_eq!(budget.bytes, total);
}

#[test]
fn arithmetic_overflow_fails_closed() {
    let mut budget = Budget {
        bytes: usize::MAX,
        limit: usize::MAX,
    };
    let error = budget
        .alias("main.veac", "value", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
    assert_eq!(budget.bytes, usize::MAX);
}
