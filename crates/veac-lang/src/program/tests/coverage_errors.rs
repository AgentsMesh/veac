use super::{empty_entry, error_code};
use crate::program::{check_source, compile_source};

fn check_error(source: &str) -> &'static str {
    check_source("invalid.veac", source).unwrap_err().as_slice()[0].code
}

#[test]
fn surface_parser_rejects_each_malformed_declaration_shape() {
    for (source, expected) in [
        ("const time 1 = 1s; project p {}", "PROGRAM_EXPECTED_WORD"),
        (
            "import 42 as value; project p {}",
            "PROGRAM_EXPECTED_STRING",
        ),
        (
            "const time value 1s; project p {}",
            "PROGRAM_EXPECTED_TOKEN",
        ),
        (
            "const time value = ; project p {}",
            "PROGRAM_EMPTY_EXPRESSION",
        ),
        (
            "const mystery value = 1; project p {}",
            "PROGRAM_VALUE_TYPE",
        ),
        (
            "preset mystery value {} project p {}",
            "PROGRAM_PRESET_KIND",
        ),
        (
            "component sequence c { slot mystery x; body {} } project p {}",
            "PROGRAM_SLOT_KIND",
        ),
        (
            "component sequence c { unknown; body {} } project p {}",
            "PROGRAM_COMPONENT_MEMBER",
        ),
        ("component sequence c { body {", "PROGRAM_UNCLOSED_BLOCK"),
    ] {
        assert_eq!(check_error(source), expected, "{source}");
    }
}

#[test]
fn file_and_instance_structure_errors_are_stable() {
    for (source, expected) in [
        ("project first {} project second {}", "PROGRAM_DUPLICATE_PROJECT"),
        ("const scalar value = 1;", "PROGRAM_PROJECT_REQUIRED"),
        ("module {} trailing", "PROGRAM_TRAILING_DECLARATION"),
        (
            "component sequence c { param scalar x; body {} }\ninstance sequence i from c { bind x 1; bind x 2; }\nproject p {}",
            "PROGRAM_DUPLICATE_BINDING",
        ),
        (
            "component sequence c { slot visual x; body {} }\ninstance sequence i from c { fill x {} fill x {} }\nproject p {}",
            "PROGRAM_DUPLICATE_FILL",
        ),
        (
            "component sequence c { body {} }\ninstance sequence i from c { unknown; }\nproject p {}",
            "PROGRAM_INSTANCE_MEMBER",
        ),
    ] {
        assert_eq!(check_error(source), expected, "{source}");
    }
}

#[test]
fn parameter_failures_cover_unknown_missing_type_and_expression_cases() {
    for (declarations, expected) in [
        (
            "component sequence c { body {} } instance sequence i from c { bind extra 1; }",
            "PROGRAM_PARAMETER_UNKNOWN",
        ),
        (
            "component sequence c { param scalar value; body {} } instance sequence i from c {}",
            "PROGRAM_PARAMETER_MISSING",
        ),
        (
            "component sequence c { param time value; body {} } instance sequence i from c { bind value 1; }",
            "PROGRAM_PARAMETER_TYPE",
        ),
        (
            "component sequence c { param time value; body {} } instance sequence i from c { bind value missing; }",
            "PROGRAM_PARAMETER_EXPRESSION",
        ),
        (
            "component sequence c { param time value default missing; body {} }",
            "PROGRAM_PARAMETER_EXPRESSION",
        ),
    ] {
        assert_eq!(error_code(&empty_entry(declarations)), expected);
    }
}

#[test]
fn slot_failures_cover_missing_unknown_source_and_body_reference_cases() {
    for (declarations, expected) in [
        (
            "component sequence c { slot visual picture; body {} } instance sequence i from c {}",
            "PROGRAM_SLOT_MISSING",
        ),
        (
            "component sequence c { body {} } instance sequence i from c { fill extra { source generated transparent; } }",
            "PROGRAM_SLOT_UNKNOWN",
        ),
        (
            "component sequence c { slot visual picture; body {} } instance sequence i from c { fill picture { nonsense; } }",
            "PROGRAM_SLOT_SOURCE",
        ),
        (
            "component sequence c { body { layer visual v { item x { source slot ghost; record { at 0s; duration 1s; } } } } }",
            "PROGRAM_SLOT_NOT_FILLED",
        ),
    ] {
        assert_eq!(error_code(&empty_entry(declarations)), expected);
    }
}

#[test]
fn duplicate_symbols_unknown_components_and_long_hygienic_ids_fail_closed() {
    assert_eq!(
        error_code(&empty_entry("const scalar x = 1; const scalar x = 2;")),
        "PROGRAM_DUPLICATE_SYMBOL"
    );
    assert_eq!(
        error_code(&empty_entry(
            "preset text-style x {} preset text-style x {}"
        )),
        "PROGRAM_DUPLICATE_SYMBOL"
    );
    assert_eq!(
        error_code(&empty_entry("instance sequence x from missing {}")),
        "PROGRAM_COMPONENT_NOT_FOUND"
    );
    let id = "a".repeat(128);
    let declarations = format!(
        "component sequence c {{ body {{ layer visual @x {{}} }} }} instance sequence {id} from c {{}}"
    );
    assert_eq!(
        error_code(&empty_entry(&declarations)),
        "PROGRAM_HYGIENIC_ID_LENGTH"
    );
}

#[test]
fn preset_use_errors_and_diagnostics_display_are_observable() {
    let scope = super::super::model::Scope::default();
    let missing = super::super::expand::preset_body("main.veac", "use text-style missing;", &scope)
        .unwrap_err();
    assert_eq!(missing.code, "PROGRAM_PRESET_NOT_FOUND");
    assert_eq!(
        super::super::expand::preset_uses("main.veac", "use mystery value;")
            .unwrap_err()
            .code,
        "PROGRAM_PRESET_KIND"
    );
    assert!(format!("{}", compile_source("not a project").unwrap_err()).contains("main.veac"));
}
