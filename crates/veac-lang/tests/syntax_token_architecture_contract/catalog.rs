use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const IMPLEMENTATION_MACROS: &[&str] = &[
    "impl_syntax_tokens!",
    "impl_local_syntax_tokens!",
    "impl_composite_syntax_tokens!",
];

#[test]
fn every_production_descriptor_is_consumed_by_the_catalog() {
    let root = manifest_dir().join("src");
    let catalog = catalog_source(&root.join("vocabulary"));
    let mut declarations = BTreeMap::<String, PathBuf>::new();
    for path in production_sources(&root) {
        for name in descriptor_names(&read(&path)) {
            assert!(
                declarations.insert(name.clone(), path.clone()).is_none(),
                "SyntaxToken descriptor {name} is implemented more than once"
            );
        }
    }
    assert!(
        !declarations.is_empty(),
        "descriptor discovery returned no types"
    );

    let missing = declarations
        .iter()
        .filter(|(name, _)| !catalog_consumes(&catalog, name))
        .map(|(name, path)| format!("{name} ({})", path.display()))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "SyntaxToken descriptors missing catalog consumption:\n{}",
        missing.join("\n")
    );
}

#[test]
fn descriptor_implementations_cannot_hide_behind_new_macros() {
    for path in production_sources(&manifest_dir().join("src")) {
        let source = read(&path);
        assert!(
            !source.contains("SyntaxToken for"),
            "{} implements SyntaxToken outside the shared primitive",
            path.display()
        );
        let wraps_implementation = source.contains("macro_rules!")
            && IMPLEMENTATION_MACROS
                .iter()
                .any(|marker| source.contains(marker));
        if wraps_implementation {
            let known_color_wrapper = path.ends_with("authoring/ast/output_syntax/color.rs")
                && source.matches("macro_rules!").count() == 1
                && source.contains("macro_rules! color_syntax");
            assert!(
                known_color_wrapper,
                "{} wraps a SyntaxToken primitive; declare it directly or audit it here",
                path.display()
            );
        }
    }
}

fn descriptor_names(source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_defined(source, "define_syntax_tokens!", &mut names);
    for marker in IMPLEMENTATION_MACROS {
        collect_first_argument(source, marker, &mut names);
    }
    collect_first_argument(source, "color_syntax!", &mut names);
    names
}

fn collect_defined(source: &str, marker: &str, names: &mut BTreeSet<String>) {
    for tail in invocation_tails(source, marker) {
        let mut words =
            tail.split(|character: char| !character.is_ascii_alphanumeric() && character != '_');
        if let Some(name) = words
            .by_ref()
            .find(|word| *word == "enum")
            .and_then(|_| words.find(|word| !word.is_empty()))
        {
            names.insert(name.to_owned());
        }
    }
}

fn collect_first_argument(source: &str, marker: &str, names: &mut BTreeSet<String>) {
    for tail in invocation_tails(source, marker) {
        let Some(argument) = tail.trim_start().strip_prefix('(') else {
            continue;
        };
        let path = argument
            .chars()
            .take_while(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | ':'))
            .collect::<String>();
        if let Some(name) = path.rsplit("::").find(|value| !value.is_empty()) {
            names.insert(name.to_owned());
        }
    }
}

fn invocation_tails<'a>(source: &'a str, marker: &'a str) -> Vec<&'a str> {
    let mut tails = Vec::new();
    let mut remaining = source;
    while let Some(index) = remaining.find(marker) {
        remaining = &remaining[index + marker.len()..];
        tails.push(remaining);
    }
    tails
}

fn catalog_consumes(source: &str, name: &str) -> bool {
    let needle = format!("{name}::");
    invocation_tails(source, &needle).into_iter().any(|tail| {
        let member = tail
            .chars()
            .take_while(|value| value.is_ascii_alphanumeric() || *value == '_')
            .collect::<String>();
        member == "ALL" || member == "TOKENS" || member.ends_with("_TOKENS")
    })
}

fn catalog_source(root: &Path) -> String {
    rust_sources(root)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("catalog"))
        })
        .map(|path| read(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn production_sources(root: &Path) -> Vec<PathBuf> {
    rust_sources(root)
        .into_iter()
        .filter(|path| !path.ends_with("syntax_token.rs") && !is_test_source(path))
        .collect()
}

fn is_test_source(path: &Path) -> bool {
    path.components().any(|part| part.as_os_str() == "tests")
        || path.file_stem().is_some_and(|stem| {
            let stem = stem.to_string_lossy();
            stem == "tests" || stem.ends_with("_tests")
        })
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
