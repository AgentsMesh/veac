use std::fs;
use std::path::{Path, PathBuf};

fn source(relative: &str) -> (PathBuf, String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let value = fs::read_to_string(&path).unwrap();
    (path, value)
}

fn production_sources(relative: &str) -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let mut paths = Vec::new();
    collect(&root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let value = fs::read_to_string(&path).unwrap();
            (path, value)
        })
        .collect()
}

fn collect(path: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().and_then(|value| value.to_str()) != Some("tests") {
                collect(&path, output);
            }
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if path.extension().and_then(|value| value.to_str()) == Some("rs")
            && name != "tests.rs"
            && !name.ends_with("_tests.rs")
            && name != "test_support.rs"
        {
            output.push(path);
        }
    }
}

fn executable_boundary_sources() -> Vec<(PathBuf, String)> {
    let mut values = vec![
        source("src/program/executable/lower.rs"),
        source("src/program/executable/model/execution.rs"),
    ];
    values.extend(production_sources("src/program/executable/lower"));
    values.extend(production_sources("src/program/expression/runtime"));
    values
}

#[test]
fn executable_boundary_neither_reparses_source_nor_reflects_through_json_or_names() {
    let sources = executable_boundary_sources();
    assert!(
        sources.len() > 75,
        "architecture scan unexpectedly lost coverage"
    );
    let forbidden = [
        "authoring::",
        "Document",
        "lower_document",
        "parser::",
        "lexer::",
        "serde_json",
        "lookup_name(",
        "lookup_function(",
        "lookup_method(",
    ];
    for (path, source) in sources {
        let source = source.replace("use crate::authoring::Span;", "");
        for needle in forbidden {
            assert!(
                !source.contains(needle),
                "{} crosses the executable boundary through {needle:?}",
                path.display()
            );
        }
    }
}

#[test]
fn runtime_dispatches_closed_numeric_domain_operations() {
    let (path, source) = source("src/program/expression/runtime/domain_graph/operation.rs");
    assert!(
        source.contains(".lookup_opcode(opcode)"),
        "{}",
        path.display()
    );
    assert!(
        source.contains("match contract.runtime_action()"),
        "{}",
        path.display()
    );
    assert!(!source.contains("contract.name()"), "{}", path.display());
}

#[test]
fn public_frontend_and_source_edits_keep_one_executable_source_path() {
    let (_, library) = source("src/lib.rs");
    let (_, program) = source("src/program/mod.rs");
    let (_, descriptors) = source("src/authoring/mod.rs");
    let (_, declaration_edit) = source("src/program/parser/type_declaration.rs");
    for removed in ["pub mod authoring", "format_document", "lower_document"] {
        assert!(!library.contains(removed), "public legacy API: {removed}");
    }
    for removed in ["CompiledProgram", "compile_source", "SourceFrontend"] {
        assert!(!program.contains(removed), "program legacy API: {removed}");
    }
    for removed in ["mod parser", "mod format", "mod lower", "Document"] {
        assert!(!descriptors.contains(removed), "legacy model: {removed}");
    }
    assert!(!declaration_edit.contains("wrap_fragment"));
    assert!(!declaration_edit.contains("struct SourceEdit"));
    assert!(declaration_edit.contains("lexer::lex"));
    assert!(declaration_edit.contains("Parser::new"));
}
