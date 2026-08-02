use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use veac_lang::authoring::{format_document, lower_document, parse};
use veac_lang::program::{check_source, compile_path};

fn examples_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn example_sources() -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(examples_root())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path().join("main.veac"))
        .filter(|path| path.is_file())
        .collect();
    paths.sort();
    paths
}

#[test]
fn every_example_round_trips_and_lowers_to_valid_canonical_ir() {
    let paths = example_sources();
    assert!(!paths.is_empty(), "no examples discovered");
    for path in paths {
        let compiled = compile_path(&path).unwrap_or_else(|diagnostics| {
            panic!("{} failed to compile: {diagnostics:?}", path.display())
        });
        let formatted = format_document(compiled.document());
        let reparsed = parse(&formatted).unwrap_or_else(|diagnostics| {
            panic!("{} failed after format: {diagnostics:?}", path.display())
        });
        assert_eq!(
            format_document(&reparsed),
            formatted,
            "{} is not format-idempotent",
            path.display()
        );
        let envelope = lower_document(compiled.document()).unwrap_or_else(|diagnostics| {
            panic!("{} failed to lower: {diagnostics:?}", path.display())
        });
        veac_ir::validate(&envelope)
            .unwrap_or_else(|diagnostics| panic!("{} invalid: {diagnostics:?}", path.display()));
    }
}

#[test]
fn example_directories_have_one_entry_and_only_reachable_modules() {
    for path in example_sources() {
        let directory = path.parent().unwrap();
        let mut sources = fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|value| value == "veac"))
            .collect::<Vec<_>>();
        sources.sort();
        let compiled = compile_path(&path).unwrap();
        let graph: BTreeSet<_> = compiled.sources().keys().map(String::as_str).collect();
        let files: BTreeSet<_> = sources
            .iter()
            .map(|source| source.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(
            files,
            graph,
            "{} contains an unreachable .veac module",
            directory.display()
        );
        for source in sources.into_iter().filter(|source| source != &path) {
            let text = fs::read_to_string(&source).unwrap();
            assert!(text.trim_start().starts_with("module {"));
            check_source(source.to_str().unwrap(), &text).unwrap();
        }
    }
}

#[test]
fn gallery_catalog_is_the_complete_example_source_of_truth() {
    let root = examples_root();
    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("catalog/gallery.json")).unwrap())
            .unwrap();
    let catalog_sources: BTreeSet<_> = catalog["targets"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|target| target["example"].as_str())
        .map(str::to_owned)
        .collect();
    let filesystem_sources: BTreeSet<_> = example_sources()
        .into_iter()
        .map(|path| format!("examples/{}", path.strip_prefix(&root).unwrap().display()))
        .collect();
    assert_eq!(catalog_sources, filesystem_sources);
}
