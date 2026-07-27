use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use veac_lang::authoring::{format_document, lower_document, parse};

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
        let source = fs::read_to_string(&path).unwrap();
        let document = parse(&source).unwrap_or_else(|diagnostics| {
            panic!("{} failed to parse: {diagnostics:?}", path.display())
        });
        let formatted = format_document(&document);
        let reparsed = parse(&formatted).unwrap_or_else(|diagnostics| {
            panic!("{} failed after format: {diagnostics:?}", path.display())
        });
        assert_eq!(
            format_document(&reparsed),
            formatted,
            "{} is not format-idempotent",
            path.display()
        );
        let envelope = lower_document(&document).unwrap_or_else(|diagnostics| {
            panic!("{} failed to lower: {diagnostics:?}", path.display())
        });
        veac_ir::validate(&envelope)
            .unwrap_or_else(|diagnostics| panic!("{} invalid: {diagnostics:?}", path.display()));
    }
}

#[test]
fn example_directories_have_one_canonical_source_file() {
    for path in example_sources() {
        let directory = path.parent().unwrap();
        let sources = fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|value| value == "veac")
            })
            .count();
        assert_eq!(
            sources,
            1,
            "{} must own one .veac file",
            directory.display()
        );
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
