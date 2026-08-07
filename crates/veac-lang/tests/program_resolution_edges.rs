use veac_lang::program::{prepare_source, Diagnostic};

#[path = "program_functions/support.rs"]
mod support;

fn project(declarations: &str) -> String {
    support::project_with(declarations, "1s")
}

fn error(declarations: &str) -> Diagnostic {
    prepare_source(&project(declarations))
        .unwrap_err()
        .as_slice()[0]
        .clone()
}

#[test]
fn function_resolution_rejects_reserved_duplicate_and_invalid_signatures() {
    let cases = [
        (
            "fn map(value: int) -> int { value }",
            "PROGRAM_FUNCTION_EXPRESSION",
        ),
        (
            "fn same() -> int { 1 } fn same() -> int { 2 }",
            "PROGRAM_DUPLICATE_SYMBOL",
        ),
        (
            "fn duplicate(value: int, value: int) -> int { value }",
            "PROGRAM_DUPLICATE_PARAMETER",
        ),
        (
            "fn missing(value: Missing) -> int { 0 }",
            "PROGRAM_UNKNOWN_TYPE",
        ),
        (
            "fn wrong() -> int { \"text\" }",
            "PROGRAM_FUNCTION_RETURN_TYPE",
        ),
        (
            "fn apply(callback: fn() -> int effect unknown) -> int { 0 }",
            "PROGRAM_FUNCTION_EFFECT",
        ),
    ];
    for (source, code) in cases {
        assert_eq!(error(source).code, code, "{source}");
    }
}

#[test]
fn function_parameter_limit_is_checked_at_the_first_excess_parameter() {
    let parameters = (0..=64)
        .map(|index| format!("value{index}: int"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("fn too_many({parameters}) -> int {{ 0 }}");
    assert_eq!(error(&source).code, "PROGRAM_FUNCTION_PARAMETER_LIMIT");
}

#[test]
fn type_resolution_rejects_duplicates_unknown_members_and_cycles() {
    let cases = [
        ("struct Same {} struct Same {}", "PROGRAM_DUPLICATE_TYPE"),
        ("struct Holder { value: Missing, }", "PROGRAM_UNKNOWN_TYPE"),
        (
            "struct First { next: Second, } struct Second { next: First, }",
            "TYPE_RECURSIVE_LAYOUT",
        ),
        ("enum Empty {}", "PROGRAM_ENUM_EMPTY"),
    ];
    for (source, code) in cases {
        assert_eq!(error(source).code, code, "{source}");
    }
}

#[test]
fn nominal_parser_rejects_the_first_member_above_its_limit() {
    let fields = (0..=64)
        .map(|index| format!("field{index}: int"))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(
        error(&format!("struct TooWide {{ {fields} }}")).code,
        "PROGRAM_TYPE_MEMBER_LIMIT"
    );
}

#[test]
fn implementation_resolution_requires_nominal_local_unique_methods() {
    let missing_identity = error("struct Timing {} impl Timing {}");
    assert_eq!(missing_identity.code, "PROGRAM_EXPECTED_LOCAL_ID");

    let invalid_identity = error("struct Timing {} impl Timing @bad.identity {}");
    assert_eq!(invalid_identity.code, "PROGRAM_IDENTIFIER");

    let primitive = error("impl time @primitive { fn value(self) -> time { self } }");
    assert_eq!(primitive.code, "PROGRAM_IMPL_TARGET");

    let duplicate = error(
        "struct Timing {} impl Timing @timing { fn value(self) -> int { 1 } fn value(self) -> int { 2 } }",
    );
    assert!(duplicate.code.contains("METHOD"), "{}", duplicate.code);
}

#[test]
fn value_namespace_rejects_duplicate_constants() {
    let duplicate = error("const int shared = 1; const int shared = 2;");
    assert_eq!(duplicate.code, "PROGRAM_DUPLICATE_SYMBOL");
}
