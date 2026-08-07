use super::super::DomainOpsetSpec;

#[test]
fn default_domain_opset_is_current_and_self_validating() {
    let value = DomainOpsetSpec::default();
    assert_eq!(value, DomainOpsetSpec::current());
    value.validate().unwrap();
}

#[test]
fn domain_validation_rejects_version_and_type_inventory_drift() {
    let mut version = DomainOpsetSpec::current();
    version.version = version.version.wrapping_add(1);
    assert_invalid(&version, "version does not match");

    let mut missing = DomainOpsetSpec::current();
    missing.types.pop();
    assert_invalid(&missing, "every closed domain type");

    let mut reordered = DomainOpsetSpec::current();
    reordered.types.swap(0, 1);
    assert_invalid(&reordered, "ordered by opcode");

    let mut duplicate_name = DomainOpsetSpec::current();
    duplicate_name.types[1].name = duplicate_name.types[0].name.clone();
    assert_invalid(&duplicate_name, "type names are not unique");
}

#[test]
fn domain_validation_rejects_unknown_and_stale_types() {
    let mut unknown = DomainOpsetSpec::current();
    unknown.types.last_mut().unwrap().opcode = u16::MAX;
    assert_invalid(&unknown, "unknown domain type opcode");

    let mut stale_name = DomainOpsetSpec::current();
    stale_name.types[0].name.push_str("Changed");
    assert_invalid(&stale_name, "stale contract");

    let mut stale_container = DomainOpsetSpec::current();
    stale_container.types[0].container = !stale_container.types[0].container;
    assert_invalid(&stale_container, "stale contract");
}

#[test]
fn domain_validation_rejects_unknown_and_duplicate_operations() {
    let mut duplicate_name = DomainOpsetSpec::current();
    duplicate_name.operations[1].name = duplicate_name.operations[0].name.clone();
    assert_invalid(&duplicate_name, "operation names are not unique");

    let mut unknown = DomainOpsetSpec::current();
    unknown.operations.last_mut().unwrap().opcode = u16::MAX;
    assert_invalid(&unknown, "unknown domain operation opcode");
}

fn assert_invalid(value: &DomainOpsetSpec, expected: &str) {
    let error = value.validate().unwrap_err().to_string();
    assert!(error.contains(expected), "unexpected error: {error}");
}
