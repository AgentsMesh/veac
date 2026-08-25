use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn production_paths_use_syntax_document_slices() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/program");
    let model = read(&root.join("model.rs"));
    assert!(model.contains("pub syntax: SyntaxDocument"));
    assert!(!model.contains("pub source: String"));
    for relative in [
        "parser/file.rs",
        "parser/function.rs",
        "parser/method.rs",
        "parser/temporal.rs",
        "index/structural.rs",
    ] {
        let source = read(&root.join(relative));
        assert!(
            !source.contains("lexer::lex("),
            "{relative} must not lex a parsed file again"
        );
    }
}

#[test]
fn lossless_document_files_stay_small_and_have_explicit_contract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/program");
    for relative in ["syntax_document.rs", "lexer/trivia.rs", "parser/slice.rs"] {
        let path = root.join(relative);
        let lines = read(&path).lines().count();
        assert!(lines < 200, "{} has {lines} lines", path.display());
    }
    let docs = read(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/language-design/executable-frontend.md"),
    );
    assert!(docs.contains("Lossless Syntax Contract"));
    assert!(docs.contains("byte-identical"));
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
