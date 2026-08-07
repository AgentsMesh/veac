use veac_lang::program::expression::{ExecutionBudget, ExecutionLimits, ResourceDelta};
use veac_lang::program::{prepare_source, Diagnostics};

use super::support::SOLID_SOURCE;

const PULSE: &str = "fn pulse(value: scalar) -> scalar { clamp(value * 2.0, 0.0, 1.0) }\n\n";

const FIRST: &str = r#"animate visual-opacity on clip(@demo, @main, @visual, @first) {
  if "aaaaaaaa" + "bbbbbbbb" == "never" { 0.0 } else { pulse(progress) }
}

"#;

const SECOND: &str = r#"animate visual-rotation on clip(@demo, @main, @visual, @first) {
  if "cccccccc" + "dddddddd" == "never" { 0deg } else { progress * 90deg }
}

"#;

#[test]
fn authored_leaves_share_one_aggregate_residual_ledger() {
    let base = succeed(SOLID_SOURCE);
    let first = succeed(&single_first());
    let second = succeed(&single_second());
    let combined = succeed(&paired());
    assert_eq!(combined.1.fuel, first.1.fuel + second.1.fuel - base.1.fuel);
    assert_eq!(
        combined.1.value_bytes,
        first.1.value_bytes + second.1.value_bytes - base.1.value_bytes
    );
    assert!(combined.1.fuel > first.1.fuel);
    assert!(combined.1.value_bytes > first.1.value_bytes);
    assert_eq!(
        combined.1.residual_nodes,
        first.1.residual_nodes + second.1.residual_nodes
    );
    assert_eq!(
        combined.1.residual_bytes,
        first.1.residual_bytes + second.1.residual_bytes
    );
    assert!(combined.1.residual_nodes > first.1.residual_nodes);
    assert!(combined.1.residual_bytes > combined.1.residual_nodes * 64);
}

#[test]
fn aggregate_exhaustion_rolls_back_graph_and_residual_charges() {
    let source = paired();
    let required = succeed(&source).1;
    assert_rollback(
        &source,
        required.fuel,
        |value, limit| value.fuel = limit,
        "fuel",
    );
    assert_rollback(
        &source,
        required.value_bytes,
        |value, limit| value.value_bytes = limit,
        "evaluated value byte",
    );
    assert_rollback(
        &source,
        required.residual_nodes,
        |value, limit| value.residual_nodes = limit,
        "residual program node",
    );
    assert_rollback(
        &source,
        required.residual_bytes,
        |value, limit| value.residual_bytes = limit,
        "residual program byte",
    );
}

fn assert_rollback(
    source: &str,
    required: usize,
    set_limit: fn(&mut ExecutionLimits, usize),
    resource: &str,
) {
    let mut limits = ExecutionLimits::default();
    set_limit(&mut limits, required - 1);
    let build = prepare_source(source).unwrap();
    let ledger = ExecutionBudget::with_resource_limits(limits);

    let first = build.execute_with_ledger(&ledger).unwrap_err();
    assert_limit(&first, resource);
    assert_eq!(ledger.usage(), ResourceDelta::default());
    let second = build.execute_with_ledger(&ledger).unwrap_err();
    assert_eq!(first.as_slice(), second.as_slice());
    assert_eq!(ledger.usage(), ResourceDelta::default());
    let recovery = prepare_source(&single_first()).unwrap();
    recovery.execute_with_ledger(&ledger).unwrap();
    assert!(ledger.usage().residual_nodes > 0);
}

#[test]
fn successful_accounting_and_output_are_byte_deterministic() {
    let source = paired();
    let first = succeed(&source);
    let second = succeed(&source);
    assert_eq!(first.0, second.0);
    assert_eq!(first.1, second.1);
    assert!(first.1.fuel > first.1.residual_nodes);
    assert!(first.1.emitted_entities > 0);
}

fn succeed(source: &str) -> (String, ResourceDelta) {
    let build = prepare_source(source).unwrap();
    let ledger = ExecutionBudget::default();
    let built = build.execute_with_ledger(&ledger).unwrap();
    (
        veac_ir::canonical_json(built.envelope()).unwrap(),
        ledger.usage(),
    )
}

fn single_second() -> String {
    format!("{SECOND}{SOLID_SOURCE}")
}

fn single_first() -> String {
    format!("{PULSE}{FIRST}{SOLID_SOURCE}")
}

fn paired() -> String {
    format!("{PULSE}{FIRST}{SECOND}{SOLID_SOURCE}")
}

fn assert_limit(error: &Diagnostics, resource: &str) {
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(diagnostic.message.contains("EXPRESSION_EXECUTION_LIMIT"));
    assert!(diagnostic.message.contains(resource), "{diagnostic:?}");
}
