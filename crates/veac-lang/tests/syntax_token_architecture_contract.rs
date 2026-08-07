use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "syntax_token_architecture_contract/catalog.rs"]
mod catalog;

#[test]
fn syntax_token_has_one_crate_level_definition() {
    let sources = rust_sources(&manifest_dir().join("src"));
    let definitions = sources
        .iter()
        .filter(|path| read(path).contains("pub trait SyntaxToken"))
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert!(definitions[0].ends_with("syntax_token.rs"));

    let lib = read(&manifest_dir().join("src/lib.rs"));
    let authoring = read(&manifest_dir().join("src/authoring/ast/mod.rs"));
    assert!(lib.contains("pub use syntax_token::SyntaxToken"));
    assert!(authoring.contains("pub use crate::SyntaxToken"));
    assert!(!authoring.contains("mod syntax_token"));
}

#[test]
fn output_keyword_islands_cannot_return() {
    for path in rust_sources(&manifest_dir().join("src")) {
        let source = read(&path);
        for forbidden in ["OutputKeyword", "output_keywords!", ".token()"] {
            assert!(
                !source.contains(forbidden),
                "{} contains {forbidden}",
                path.display()
            );
        }
    }
}

#[test]
fn program_token_types_use_the_shared_declaration_primitive() {
    for relative in [
        "src/program/expression/builtin.rs",
        "src/program/expression/unit.rs",
        "src/program/expression/value.rs",
    ] {
        let path = manifest_dir().join(relative);
        let source = read(&path);
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            source.contains("define_syntax_tokens!"),
            "{}",
            path.display()
        );
        assert!(
            !normalized.contains("Self::ALL.into_iter().find"),
            "{} mirrors parse",
            path.display()
        );
        assert!(
            !source.contains("pub fn parse(value"),
            "{} mirrors parse",
            path.display()
        );
    }
}

#[test]
fn output_and_selection_tokens_use_shared_implementation_macros() {
    for relative in [
        "src/authoring/ast/output_syntax.rs",
        "src/authoring/ast/output_syntax/color.rs",
        "src/authoring/ast/output_syntax/delivery.rs",
    ] {
        let source = read(&manifest_dir().join(relative));
        assert!(source.contains("impl_local_syntax_tokens!"));
    }
    let selections = read(&manifest_dir().join("src/authoring/ast/output_selection.rs"));
    assert!(selections.contains("define_syntax_tokens!"));
    assert_eq!(
        selections.matches("impl_composite_syntax_tokens!").count(),
        2
    );
    assert!(!selections.contains("match OutputSentinel::parse"));
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_owned()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|value| value == "rs") {
                files.push(path);
            }
        }
    }
    files
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
