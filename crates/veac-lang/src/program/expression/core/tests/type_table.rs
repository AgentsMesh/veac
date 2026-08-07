use super::{raw, verify_error};
use crate::program::expression::{
    CoreType, CoreTypeEntry, CoreTypeId, FunctionMap, TypeEnvironment, ValueType, CORE_VERSION,
};

#[test]
fn lowering_uses_child_first_deduplicated_recursive_types() {
    assert_eq!(table("int"), ["int"]);
    assert_eq!(table("list<int>"), ["int", "list<int>"]);
    assert_eq!(table("range<int>"), ["int", "range<int>"]);
    assert_eq!(
        table("map<text, list<int>>"),
        ["text", "int", "list<int>", "map<text, list<int>>"]
    );
    assert_eq!(
        table("(int, list<int>, int)"),
        ["int", "list<int>", "(int, list<int>, int)"]
    );
    assert_eq!(
        source_table(
            "fn(values: list<int>, label: text) -> map<identifier, time> effect pure { \
             #{identifier(\"result\"): 1s} }",
        ),
        [
            "int",
            "list<int>",
            "text",
            "identifier",
            "time",
            "map<identifier, time>",
            "fn(list<int>, text) -> map<identifier, time> effect pure",
        ]
    );
}

fn source_table(source: &str) -> Vec<String> {
    raw(source)
        .0
        .types
        .entries()
        .iter()
        .map(|entry| entry.kind().value_type().unwrap().to_string())
        .collect()
}

#[test]
fn rejects_non_dense_duplicate_and_unknown_type_ids() {
    let (mut dense, functions) = raw("1");
    dense.types.entries[0].id = CoreTypeId::new(1);
    assert_message(dense, &functions, "Core type IDs must be dense");

    let (mut duplicate, functions) = raw("1");
    duplicate.types.entries.push(CoreTypeEntry {
        id: CoreTypeId::new(1),
        kind: duplicate.types.entries[0].kind.clone(),
    });
    assert_message(
        duplicate,
        &functions,
        "Core type table contains a duplicate type",
    );

    let (mut unknown, functions) = raw("1");
    unknown.blocks[0].instructions[0].type_id = CoreTypeId::new(99);
    assert_message(unknown, &functions, "Core value references an unknown type");
}

#[test]
fn rejects_child_after_parent_noncanonical_order_and_unused_types() {
    let (mut child_after_parent, functions) = program("list<int>");
    let (first, second) = kinds(&mut child_after_parent);
    std::mem::swap(first, second);
    assert_message(
        child_after_parent,
        &functions,
        "Core semantic child type must precede its parent",
    );

    let (mut order, functions) = program("(int, time)");
    let (first, second) = kinds(&mut order);
    std::mem::swap(first, second);
    assert_message(
        order,
        &functions,
        "Core type table is not in first-use order",
    );

    let (mut unused, functions) = raw("1");
    unused.types.entries.push(CoreTypeEntry {
        id: CoreTypeId::new(1),
        kind: CoreType::Value(ValueType::parse("time").unwrap()),
    });
    assert_message(
        unused,
        &functions,
        "Core type table contains an unused type",
    );
}

#[test]
fn rejects_invalid_internal_map_tokens_and_public_positions() {
    let (mut non_map, functions) = raw("1");
    non_map.types.entries.push(CoreTypeEntry {
        id: CoreTypeId::new(1),
        kind: CoreType::MapBuilder {
            map_type: CoreTypeId::new(0),
        },
    });
    assert_message(
        non_map,
        &functions,
        "Core map token must reference a map type",
    );

    let (mut public, functions) = program("map<text, int>");
    let token = CoreTypeId::new(public.types.entries.len() as u32);
    public.types.entries.push(CoreTypeEntry {
        id: token,
        kind: CoreType::MapPending {
            map_type: public.result_type,
        },
    });
    public.result_type = token;
    assert_message(
        public,
        &functions,
        "Core position requires a known value type",
    );
}

#[test]
fn previous_core_version_is_rejected_and_current_version_is_accepted() {
    let (current, functions) = raw("1");
    assert_eq!(current.version, CORE_VERSION);
    assert!(super::super::verify(current.clone(), functions.registry(), &[]).is_ok());
    let mut old = current;
    old.version = CORE_VERSION - 1;
    assert_message(
        old,
        &functions,
        &format!("unsupported Core version {}", CORE_VERSION - 1),
    );
}

fn table(value_type: &str) -> Vec<String> {
    program(value_type)
        .0
        .types
        .entries()
        .iter()
        .map(|entry| entry.kind().value_type().unwrap().to_string())
        .collect()
}

fn program(value_type: &str) -> (super::super::CoreProgram, FunctionMap) {
    let types = [("input".to_owned(), ValueType::parse(value_type).unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    super::raw_with_types("input", &types)
}

fn kinds(program: &mut super::super::CoreProgram) -> (&mut CoreType, &mut CoreType) {
    let (first, rest) = program.types.entries.split_at_mut(1);
    (&mut first[0].kind, &mut rest[0].kind)
}

fn assert_message(program: super::super::CoreProgram, functions: &FunctionMap, message: &str) {
    assert_eq!(verify_error(program, functions).message(), message);
}
