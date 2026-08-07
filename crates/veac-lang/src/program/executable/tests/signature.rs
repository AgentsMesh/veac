use super::{error, MAIN};

#[test]
fn executable_entry_requires_one_root_local_main() {
    let source = "fn helper(context: Context) -> Project { context }";
    let missing = error(source);
    assert_eq!(missing.code, "PROGRAM_EXECUTABLE_MAIN_MISSING");
    assert_eq!(missing.span.start, source.len());
    assert_eq!(missing.span.end, source.len());

    let source = format!("{MAIN}\n{MAIN}");
    let duplicate = error(&source);
    assert_eq!(duplicate.code, "PROGRAM_EXECUTABLE_MAIN_DUPLICATE");
    assert_eq!(
        &source[duplicate.span.start..duplicate.span.start + 2],
        "fn"
    );
}

#[test]
fn imported_main_does_not_satisfy_the_root_contract() {
    let entry = crate::program::LoadedSource {
        id: "main.veac".to_owned(),
        source: "import \"library.veac\" as library;".to_owned(),
    };
    let loader = crate::program::loader::MemoryLoader::new(std::collections::BTreeMap::from([(
        "library.veac".to_owned(),
        "module { export fn main(context: Context) -> Project { context } }".to_owned(),
    )]));
    let diagnostics = super::super::prepare_with_loader(entry, &loader).unwrap_err();
    assert_eq!(
        diagnostics.as_slice()[0].code,
        "PROGRAM_EXECUTABLE_MAIN_MISSING"
    );
}

#[test]
fn executable_main_arity_name_and_types_have_stable_diagnostics() {
    let cases = [
        (
            "fn main() -> Project { project(identifier(\"p\"), canvas(1px, 1px), fps(1)) }",
            "PROGRAM_EXECUTABLE_MAIN_ARITY",
            "fn main",
        ),
        (
            "fn main(input: Context) -> Project { input }",
            "PROGRAM_EXECUTABLE_MAIN_PARAMETER_NAME",
            "input: Context",
        ),
        (
            "fn main(context: Project) -> Project { context }",
            "PROGRAM_EXECUTABLE_MAIN_PARAMETER_TYPE",
            "Project",
        ),
        (
            "fn main(context: Context) -> Sequence { sequence(identifier(\"s\")) }",
            "PROGRAM_EXECUTABLE_MAIN_RETURN_TYPE",
            "Sequence",
        ),
    ];
    for (source, code, authored) in cases {
        let diagnostic = error(source);
        assert_eq!(diagnostic.code, code);
        assert!(
            source[diagnostic.span.start..diagnostic.span.end].contains(authored),
            "{code} did not retain its authored span"
        );
    }
}
