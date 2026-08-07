use super::super::{DomainOpsetSpec, StandardLibrarySpec};

#[test]
fn default_standard_library_is_current_and_valid() {
    let value = StandardLibrarySpec::default();
    assert_eq!(value, StandardLibrarySpec::current());
    value.validate(&DomainOpsetSpec::current()).unwrap();
}

#[test]
fn standard_library_rejects_version_inventory_and_type_order_drift() {
    let opset = DomainOpsetSpec::current();
    let mut version = StandardLibrarySpec::current();
    version.domain_opset_version = version.domain_opset_version.wrapping_add(1);
    assert_invalid(&version, &opset, "different domain opset");

    let mut missing = StandardLibrarySpec::current();
    missing.types.pop();
    assert_invalid(&missing, &opset, "publish every domain type");

    let mut reordered = StandardLibrarySpec::current();
    reordered.types.swap(0, 1);
    assert_invalid(&reordered, &opset, "types are not unique and ordered");
}

#[test]
fn standard_library_rejects_unknown_and_mismatched_types() {
    let opset = DomainOpsetSpec::current();
    let mut unknown = StandardLibrarySpec::current();
    unknown.types[0].domain_type_opcode = u16::MAX;
    assert_invalid(&unknown, &opset, "unknown domain opcode");

    let mut mismatch = StandardLibrarySpec::current();
    mismatch.types[0].domain_type_opcode = mismatch.types[1].domain_type_opcode;
    assert_invalid(&mismatch, &opset, "name and opcode disagree");
}

#[test]
fn standard_library_rejects_noncanonical_callable_order() {
    let opset = DomainOpsetSpec::current();
    let mut functions = StandardLibrarySpec::current();
    functions.free_functions.swap(0, 1);
    assert_invalid(&functions, &opset, "functions are not unique and ordered");

    let mut methods = StandardLibrarySpec::current();
    methods.methods.swap(0, 1);
    assert_invalid(&methods, &opset, "methods are not unique");
}

#[test]
fn standard_library_rejects_unknown_functions_and_methods() {
    let opset = DomainOpsetSpec::current();
    let mut function = StandardLibrarySpec::current();
    function.free_functions[0].name = "!unknown".into();
    assert_invalid(&function, &opset, "unknown standard function");

    let mut receiver = StandardLibrarySpec::current();
    receiver.methods.last_mut().unwrap().receiver.opcode = u16::MAX;
    assert_invalid(&receiver, &opset, "unknown receiver opcode");

    let mut method = StandardLibrarySpec::current();
    method.methods[0].name = "!unknown".into();
    assert_invalid(&method, &opset, "unknown standard method");
}

fn assert_invalid(value: &StandardLibrarySpec, opset: &DomainOpsetSpec, expected: &str) {
    let error = value.validate(opset).unwrap_err().to_string();
    assert!(error.contains(expected), "unexpected error: {error}");
}
